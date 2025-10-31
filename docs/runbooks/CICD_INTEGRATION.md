# CI/CD Integration Runbook

## Overview

Integrate Weaver validation into CI/CD pipelines as a deployment gate.

## GitHub Actions

### Complete Workflow

```yaml
# .github/workflows/weaver-validation.yml
name: Weaver Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY_PATH: registry/
  RUST_VERSION: 1.70

jobs:
  schema-validation:
    name: Schema Validation
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Weaver
        run: |
          cargo install weaver-cli
          weaver --version

      - name: Validate Schema
        run: |
          weaver registry check --registry ${{ env.REGISTRY_PATH }}

      - name: Upload Schema Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: schema-report
          path: schema-check.log

  build-and-test:
    name: Build and Test
    runs-on: ubuntu-latest
    needs: schema-validation

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ env.RUST_VERSION }}
          profile: minimal
          override: true

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --release --features otel

      - name: Run Unit Tests
        run: cargo test --lib --features otel

  live-validation:
    name: Live Weaver Validation
    runs-on: ubuntu-latest
    needs: build-and-test

    services:
      jaeger:
        image: jaegertracing/all-in-one:latest
        ports:
          - 4317:4317
          - 16686:16686

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ env.RUST_VERSION }}

      - name: Install Weaver
        run: cargo install weaver-cli

      - name: Start Weaver Live-Check
        run: |
          weaver registry live-check \
            --registry ${{ env.REGISTRY_PATH }} \
            --otlp-grpc-port 4318 \
            --admin-port 8080 \
            --output ./validation_output \
            --format json \
            --no-stream &
          echo $! > weaver.pid
          sleep 5

      - name: Run Tests with Telemetry
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4318
          RUST_LOG: debug
        run: |
          cargo test --features otel --workspace -- --test-threads=1

      - name: Stop Weaver and Get Report
        if: always()
        run: |
          if [ -f weaver.pid ]; then
            kill -HUP $(cat weaver.pid) || true
            sleep 2
          fi

      - name: Parse Validation Report
        if: always()
        run: |
          if [ -f validation_output/validation_report.json ]; then
            violations=$(jq '.violations' validation_output/validation_report.json)
            echo "violations=$violations" >> $GITHUB_OUTPUT
            echo "### Weaver Validation Report" >> $GITHUB_STEP_SUMMARY
            echo "" >> $GITHUB_STEP_SUMMARY
            echo "**Violations:** $violations" >> $GITHUB_STEP_SUMMARY
            echo "**Coverage:** $(jq '.registry_coverage * 100' validation_output/validation_report.json)%" >> $GITHUB_STEP_SUMMARY

            if [ "$violations" -gt 0 ]; then
              echo "::error::Weaver detected $violations violations"
              exit 1
            fi
          else
            echo "::warning::Validation report not found"
          fi

      - name: Upload Validation Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: validation-report
          path: validation_output/

  deployment-gate:
    name: Deployment Gate
    runs-on: ubuntu-latest
    needs: live-validation
    if: github.ref == 'refs/heads/main'

    steps:
      - name: Download Validation Report
        uses: actions/download-artifact@v3
        with:
          name: validation-report

      - name: Verify Zero Violations
        run: |
          violations=$(jq '.violations' validation_report.json)
          if [ "$violations" -gt 0 ]; then
            echo "::error::Cannot deploy with $violations violations"
            exit 1
          fi
          echo "::notice::✅ Validation passed - deployment approved"

      - name: Tag Release
        if: success()
        run: |
          git tag -a v${{ github.run_number }} -m "Validated release"
          git push origin v${{ github.run_number }}
```

## GitLab CI

```yaml
# .gitlab-ci.yml
variables:
  REGISTRY_PATH: registry/
  DOCKER_DRIVER: overlay2

stages:
  - validate-schema
  - build
  - test
  - validate-live
  - deploy

schema-validation:
  stage: validate-schema
  image: rust:1.70
  script:
    - cargo install weaver-cli
    - weaver registry check --registry $REGISTRY_PATH
  artifacts:
    reports:
      dotenv: schema-check.env
    when: always

build:
  stage: build
  image: rust:1.70
  script:
    - cargo build --release --features otel
  artifacts:
    paths:
      - target/release/clnrm
    expire_in: 1 hour

unit-tests:
  stage: test
  image: rust:1.70
  script:
    - cargo test --lib --features otel
  coverage: '/^\s*lines:\s*\d+\.\d+\%/'

weaver-live-check:
  stage: validate-live
  image: rust:1.70
  services:
    - name: jaegertracing/all-in-one:latest
      alias: jaeger
  before_script:
    - cargo install weaver-cli
  script:
    - |
      weaver registry live-check \
        --registry $REGISTRY_PATH \
        --otlp-grpc-port 4317 \
        --output validation_output \
        --format json &
      WEAVER_PID=$!
      sleep 5

    - |
      OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
      cargo test --features otel --workspace

    - |
      kill -HUP $WEAVER_PID || true
      sleep 2

    - |
      if [ -f validation_output/validation_report.json ]; then
        violations=$(jq '.violations' validation_output/validation_report.json)
        if [ "$violations" -gt 0 ]; then
          echo "❌ Validation failed with $violations violations"
          exit 1
        fi
      fi
  artifacts:
    paths:
      - validation_output/
    reports:
      junit: validation_output/junit.xml
    when: always

deployment-gate:
  stage: deploy
  only:
    - main
  script:
    - |
      violations=$(jq '.violations' validation_output/validation_report.json)
      if [ "$violations" -eq 0 ]; then
        echo "✅ Deployment approved"
        # Deploy logic here
      else
        echo "❌ Deployment blocked"
        exit 1
      fi
```

## Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any

    environment {
        REGISTRY_PATH = 'registry/'
        OTEL_ENDPOINT = 'http://localhost:4317'
    }

    stages {
        stage('Schema Validation') {
            steps {
                sh 'cargo install weaver-cli'
                sh "weaver registry check --registry ${REGISTRY_PATH}"
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release --features otel'
            }
        }

        stage('Weaver Live Validation') {
            steps {
                script {
                    // Start Weaver
                    sh """
                        weaver registry live-check \\
                          --registry ${REGISTRY_PATH} \\
                          --otlp-grpc-port 4317 \\
                          --output validation_output \\
                          --format json &
                        echo \$! > weaver.pid
                        sleep 5
                    """

                    try {
                        // Run tests
                        sh """
                            OTEL_EXPORTER_OTLP_ENDPOINT=${OTEL_ENDPOINT} \\
                            cargo test --features otel --workspace
                        """
                    } finally {
                        // Stop Weaver
                        sh '''
                            if [ -f weaver.pid ]; then
                                kill -HUP $(cat weaver.pid) || true
                                sleep 2
                            fi
                        '''
                    }

                    // Check report
                    def report = readJSON file: 'validation_output/validation_report.json'
                    if (report.violations > 0) {
                        error("Weaver detected ${report.violations} violations")
                    }
                }
            }
        }

        stage('Deployment Gate') {
            when {
                branch 'main'
            }
            steps {
                script {
                    def report = readJSON file: 'validation_output/validation_report.json'

                    if (report.violations == 0) {
                        echo "✅ Validation passed - deploying"
                        // Deployment logic
                    } else {
                        error("Deployment blocked - violations detected")
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'validation_output/**', allowEmptyArchive: true
            publishHTML([
                reportDir: 'validation_output',
                reportFiles: 'validation_report.json',
                reportName: 'Weaver Validation Report'
            ])
        }
    }
}
```

## CircleCI

```yaml
# .circleci/config.yml
version: 2.1

executors:
  rust-executor:
    docker:
      - image: rust:1.70
    working_directory: ~/clnrm

jobs:
  schema-validation:
    executor: rust-executor
    steps:
      - checkout
      - run:
          name: Install Weaver
          command: cargo install weaver-cli
      - run:
          name: Validate Schema
          command: weaver registry check --registry registry/

  build-and-test:
    executor: rust-executor
    steps:
      - checkout
      - restore_cache:
          keys:
            - cargo-{{ checksum "Cargo.lock" }}
      - run:
          name: Build
          command: cargo build --release --features otel
      - save_cache:
          key: cargo-{{ checksum "Cargo.lock" }}
          paths:
            - ~/.cargo
            - target/
      - run:
          name: Unit Tests
          command: cargo test --lib --features otel

  weaver-validation:
    executor: rust-executor
    docker:
      - image: rust:1.70
      - image: jaegertracing/all-in-one:latest
    steps:
      - checkout
      - run:
          name: Install Weaver
          command: cargo install weaver-cli
      - run:
          name: Start Weaver
          command: |
            weaver registry live-check \
              --registry registry/ \
              --otlp-grpc-port 4317 \
              --output validation_output \
              --format json &
            echo $! > weaver.pid
            sleep 5
          background: true
      - run:
          name: Run Tests
          environment:
            OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4317
          command: cargo test --features otel --workspace
      - run:
          name: Stop Weaver
          when: always
          command: |
            if [ -f weaver.pid ]; then
              kill -HUP $(cat weaver.pid) || true
            fi
      - run:
          name: Check Report
          command: |
            violations=$(jq '.violations' validation_output/validation_report.json)
            if [ "$violations" -gt 0 ]; then
              echo "Validation failed"
              exit 1
            fi
      - store_artifacts:
          path: validation_output/

workflows:
  version: 2
  validate-and-deploy:
    jobs:
      - schema-validation
      - build-and-test:
          requires:
            - schema-validation
      - weaver-validation:
          requires:
            - build-and-test
      - deployment-approval:
          type: approval
          requires:
            - weaver-validation
          filters:
            branches:
              only: main
```

## Azure DevOps

```yaml
# azure-pipelines.yml
trigger:
  - main
  - develop

pool:
  vmImage: 'ubuntu-latest'

variables:
  REGISTRY_PATH: 'registry/'

stages:
  - stage: Validate
    jobs:
      - job: SchemaValidation
        steps:
          - script: |
              cargo install weaver-cli
              weaver registry check --registry $(REGISTRY_PATH)
            displayName: 'Validate Schema'

  - stage: Build
    dependsOn: Validate
    jobs:
      - job: BuildAndTest
        steps:
          - script: cargo build --release --features otel
            displayName: 'Build'
          - script: cargo test --lib --features otel
            displayName: 'Unit Tests'

  - stage: LiveValidation
    dependsOn: Build
    jobs:
      - job: WeaverValidation
        services:
          jaeger:
            image: jaegertracing/all-in-one:latest
            ports:
              - 4317:4317
        steps:
          - script: |
              cargo install weaver-cli
              weaver registry live-check \
                --registry $(REGISTRY_PATH) \
                --otlp-grpc-port 4317 \
                --output validation_output \
                --format json &
              echo $! > weaver.pid
              sleep 5
            displayName: 'Start Weaver'

          - script: |
              export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
              cargo test --features otel --workspace
            displayName: 'Run Tests'

          - script: |
              if [ -f weaver.pid ]; then
                kill -HUP $(cat weaver.pid) || true
              fi
            displayName: 'Stop Weaver'
            condition: always()

          - task: PublishBuildArtifacts@1
            inputs:
              pathToPublish: 'validation_output'
              artifactName: 'validation-report'

  - stage: Deploy
    dependsOn: LiveValidation
    condition: and(succeeded(), eq(variables['Build.SourceBranch'], 'refs/heads/main'))
    jobs:
      - job: DeploymentGate
        steps:
          - download: current
            artifact: validation-report
          - script: |
              violations=$(jq '.violations' $(Pipeline.Workspace)/validation-report/validation_report.json)
              if [ "$violations" -gt 0 ]; then
                echo "##vso[task.logissue type=error]Deployment blocked: $violations violations"
                exit 1
              fi
              echo "##vso[task.complete result=Succeeded;]Deployment approved"
            displayName: 'Deployment Gate'
```

---

**Last Updated:** 2025-10-30
