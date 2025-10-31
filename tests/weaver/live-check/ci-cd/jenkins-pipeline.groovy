// Jenkins Pipeline Integration Example
// Demonstrates Weaver live-check for continuous validation

pipeline {
    agent any

    environment {
        OTEL_EXPORTER_OTLP_ENDPOINT = 'http://localhost:4318'
        OTEL_SERVICE_NAME = 'clnrm-jenkins'
        WEAVER_BIN = '/usr/local/bin/weaver'
    }

    stages {
        stage('Setup') {
            steps {
                script {
                    // Install Weaver if not present
                    sh '''
                        if [ ! -f ${WEAVER_BIN} ]; then
                            curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux-amd64 -o ${WEAVER_BIN}
                            chmod +x ${WEAVER_BIN}
                        fi
                        ${WEAVER_BIN} --version
                    '''
                }
            }
        }

        stage('Start OTLP Collector') {
            steps {
                sh '''
                    docker run -d \
                        --name otel-collector-${BUILD_ID} \
                        -p 4317:4317 \
                        -p 4318:4318 \
                        otel/opentelemetry-collector:latest

                    # Wait for collector to be ready
                    until curl -f http://localhost:13133/; do sleep 2; done
                '''
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release --features otel'
            }
        }

        stage('Test with Telemetry') {
            steps {
                sh '''
                    # Run tests with OTLP export
                    ./target/release/clnrm self-test --suite otel &
                    APP_PID=$!
                    echo ${APP_PID} > app.pid

                    # Give tests time to generate telemetry
                    sleep 10
                '''
            }
        }

        stage('Weaver Validation') {
            steps {
                script {
                    // Run live-check validation
                    sh '''
                        ${WEAVER_BIN} registry live-check \
                            --registry registry/ \
                            --otlp-http http://localhost:4318 \
                            --timeout 60s \
                            --output json > weaver-results.json || true
                    '''

                    // Parse results
                    def results = readJSON file: 'weaver-results.json'
                    def violations = results.violations?.size() ?: 0
                    def errors = results.violations?.count { it.severity == 'error' } ?: 0
                    def warnings = results.violations?.count { it.severity == 'warning' } ?: 0

                    echo "Weaver Validation Results:"
                    echo "  Total violations: ${violations}"
                    echo "  Errors: ${errors}"
                    echo "  Warnings: ${warnings}"

                    // Set build status
                    if (errors > 0) {
                        currentBuild.result = 'FAILURE'
                        error("Schema validation FAILED: ${errors} error(s) detected")
                    } else if (warnings > 0) {
                        currentBuild.result = 'UNSTABLE'
                        echo "Warning: ${warnings} warning(s) detected"
                    } else {
                        echo "✅ Schema validation PASSED"
                    }
                }
            }
        }
    }

    post {
        always {
            // Stop application
            sh '''
                if [ -f app.pid ]; then
                    kill -SIGTERM $(cat app.pid) || true
                fi
            '''

            // Stop OTLP collector
            sh "docker stop otel-collector-${BUILD_ID} || true"
            sh "docker rm otel-collector-${BUILD_ID} || true"

            // Archive validation report
            archiveArtifacts artifacts: 'weaver-results.json', allowEmptyArchive: true

            // Publish results
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: '.',
                reportFiles: 'weaver-results.json',
                reportName: 'Weaver Validation Report'
            ])
        }

        failure {
            emailext(
                subject: "Weaver Validation Failed: ${env.JOB_NAME} - Build #${env.BUILD_NUMBER}",
                body: """
                    Schema validation failed in build ${env.BUILD_NUMBER}.

                    Check the validation report for details:
                    ${env.BUILD_URL}artifact/weaver-results.json
                """,
                to: '${DEFAULT_RECIPIENTS}'
            )
        }
    }
}
