# CI/CD Integration

CI/CD integration enables automated testing in continuous integration and deployment pipelines. This chapter covers integrating clnrm with popular CI/CD platforms.

## Overview

clnrm integrates with major CI/CD platforms:
- **GitHub Actions** - Native GitHub integration
- **GitLab CI** - GitLab's built-in CI/CD
- **Jenkins** - Enterprise CI/CD platform
- **Azure DevOps** - Microsoft's DevOps platform
- **CircleCI** - Cloud-native CI/CD

## GitHub Actions Integration

### Basic GitHub Actions Workflow

Simple workflow for running clnrm tests:

```yaml
# .github/workflows/clnrm-tests.yml
name: Cleanroom Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Install clnrm
      run: cargo install --path .

    - name: Pull test images
      run: clnrm pull tests/

    - name: Run tests
      run: clnrm run tests/ --parallel --workers 4

    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: |
          *.json
          *.xml
          *.sha256
```

### Advanced GitHub Actions Workflow

Comprehensive workflow with matrix testing:

```yaml
# .github/workflows/advanced-clnrm-tests.yml
name: Advanced Cleanroom Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

env:
  CARGO_TERM_COLOR: always

jobs:
  # Security scanning
  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Run security tests
      run: clnrm run tests/security/ --env production

  # Matrix testing across environments
  matrix-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        environment: [test, staging, production]
        service: [api, database, cache]

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Run matrix tests
      run: |
        clnrm run tests/matrix/ \
          --env ${{ matrix.environment }} \
          --service ${{ matrix.service }}

  # Performance testing
  performance:
    runs-on: ubuntu-latest
    needs: matrix-test
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Run performance tests
      run: clnrm run tests/performance/ --baseline production

    - name: Check for regressions
      run: |
        if clnrm run tests/performance/ --check-regressions; then
          echo "✅ No performance regressions"
        else
          echo "❌ Performance regression detected"
          exit 1
        fi

  # Integration with external systems
  integration:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15-alpine
        env:
          POSTGRES_PASSWORD: testpass
          POSTGRES_DB: testdb
        ports:
          - 5432:5432

      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Run integration tests
      run: clnrm run tests/integration/ --external-services

  # Report generation and upload
  reports:
    runs-on: ubuntu-latest
    needs: [security, matrix-test, performance, integration]
    if: always()

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install clnrm
      run: cargo install --path .

    - name: Generate reports
      run: clnrm run tests/reports/ --format html,json,junit

    - name: Upload HTML report
      uses: actions/upload-artifact@v3
      with:
        name: test-report-html
        path: test-report.html

    - name: Upload JSON results
      uses: actions/upload-artifact@v3
      with:
        name: test-results-json
        path: test-results.json

    - name: Upload JUnit XML
      uses: actions/upload-artifact@v3
      with:
        name: junit-xml
        path: junit.xml
```

### GitHub Actions with Docker

Run clnrm tests in Docker containers:

```yaml
# .github/workflows/docker-clnrm-tests.yml
name: Docker Cleanroom Tests

on:
  push:
    branches: [ main ]

jobs:
  docker-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        dockerfile: [Dockerfile.test, Dockerfile.integration]

    steps:
    - uses: actions/checkout@v4

    - name: Build test image
      run: docker build -f ${{ matrix.dockerfile }} -t clnrm-test .

    - name: Run tests in container
      run: |
        docker run --rm \
          -v $(pwd):/workspace \
          -w /workspace \
          clnrm-test \
          clnrm run tests/ --parallel

    - name: Run performance tests
      run: |
        docker run --rm \
          -v $(pwd):/workspace \
          -w /workspace \
          clnrm-test \
          clnrm run tests/performance/ --baseline production
```

## GitLab CI Integration

### Basic GitLab CI Pipeline

Simple pipeline configuration:

```yaml
# .gitlab-ci.yml
stages:
  - install
  - test
  - performance
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/
    - target/

install_clnrm:
  stage: install
  image: rust:latest
  script:
    - cargo install --path .
  artifacts:
    paths:
      - $CARGO_HOME/bin/clnrm
    expire_in: 1 hour

test:unit:
  stage: test
  image: rust:latest
  dependencies:
    - install_clnrm
  script:
    - clnrm run tests/unit/
  artifacts:
    reports:
      junit: unit-test-results.xml

test:integration:
  stage: test
  image: rust:latest
  dependencies:
    - install_clnrm
  services:
    - postgres:15-alpine
    - redis:7-alpine
  script:
    - clnrm run tests/integration/ --external-services
  artifacts:
    reports:
      junit: integration-test-results.xml

performance:baseline:
  stage: performance
  image: rust:latest
  dependencies:
    - install_clnrm
  script:
    - clnrm run tests/performance/ --baseline $CI_COMMIT_REF_NAME
  only:
    - main
    - tags

deploy:smoke_tests:
  stage: deploy
  image: rust:latest
  dependencies:
    - install_clnrm
  script:
    - clnrm run tests/smoke/ --env production
  only:
    - tags
  environment:
    name: production
```

### Advanced GitLab CI Pipeline

Comprehensive pipeline with parallel execution:

```yaml
# .gitlab-ci.yml
stages:
  - validate
  - test
  - performance
  - security
  - deploy

variables:
  CLNRM_WORKERS: "4"
  CLNRM_TIMEOUT: "30"

# Template for test jobs
.test_template: &test_template
  image: rust:latest
  before_script:
    - cargo install --path .
    - clnrm pull tests/
  artifacts:
    reports:
      junit: $CI_JOB_NAME-results.xml
    paths:
      - "*.json"
      - "*.xml"

validate:toml:
  <<: *test_template
  stage: validate
  script:
    - clnrm validate tests/
  allow_failure: false

test:unit:
  <<: *test_template
  stage: test
  script:
    - clnrm run tests/unit/ --parallel --workers $CLNRM_WORKERS
  coverage: '/Coverage: \d+\.\d+%/'

test:integration:
  <<: *test_template
  stage: test
  services:
    - postgres:15-alpine
    - redis:7-alpine
  script:
    - clnrm run tests/integration/ --parallel --workers $CLNRM_WORKERS
  after_script:
    - clnrm run tests/cleanup/

test:chaos:
  <<: *test_template
  stage: test
  script:
    - clnrm run tests/chaos/ --chaos-enabled
  allow_failure: true

performance:load:
  <<: *test_template
  stage: performance
  script:
    - clnrm run tests/performance/ --baseline $CI_COMMIT_REF_NAME
  only:
    - main
    - tags

security:scan:
  <<: *test_template
  stage: security
  script:
    - clnrm run tests/security/ --security-scan
  allow_failure: false

deploy:smoke:
  <<: *test_template
  stage: deploy
  script:
    - clnrm run tests/smoke/ --env production
  only:
    - tags
  environment:
    name: production

# Parallel test execution
test:parallel:
  stage: test
  image: rust:latest
  parallel:
    matrix:
      - TEST_SUITE: [api, database, cache, integration]
  script:
    - cargo install --path .
    - clnrm run tests/$TEST_SUITE/ --parallel --workers 2
  artifacts:
    reports:
      junit: $TEST_SUITE-results.xml
```

## Jenkins Integration

### Jenkins Pipeline Script

Jenkins pipeline for clnrm:

```groovy
pipeline {
    agent any

    environment {
        CARGO_HOME = "${env.WORKSPACE}/.cargo"
        RUST_BACKTRACE = '1'
        CLNRM_WORKERS = '4'
    }

    stages {
        stage('Install Dependencies') {
            steps {
                sh '''
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    source $HOME/.cargo/env
                    cargo install --path .
                '''
            }
        }

        stage('Pull Images') {
            steps {
                sh 'clnrm pull tests/'
            }
        }

        stage('Unit Tests') {
            steps {
                sh 'clnrm run tests/unit/ --parallel --workers $CLNRM_WORKERS'
            }
            post {
                always {
                    junit 'test-results.xml'
                }
            }
        }

        stage('Integration Tests') {
            steps {
                sh '''
                    docker run -d --name postgres -e POSTGRES_PASSWORD=testpass -e POSTGRES_DB=testdb -p 5432:5432 postgres:15-alpine
                    docker run -d --name redis -p 6379:6379 redis:7-alpine
                    sleep 10
                    clnrm run tests/integration/ --external-services
                '''
            }
            post {
                always {
                    sh '''
                        docker stop postgres redis || true
                        docker rm postgres redis || true
                    '''
                }
            }
        }

        stage('Performance Tests') {
            when {
                branch 'main'
            }
            steps {
                sh 'clnrm run tests/performance/ --baseline production'
            }
        }

        stage('Deploy Tests') {
            when {
                tag pattern: 'v*', comparator: 'REGEXP'
            }
            steps {
                sh 'clnrm run tests/smoke/ --env production'
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: '*.json,*.xml,*.sha256'
            cleanWs()
        }
    }
}
```

### Jenkins with Docker Agents

Use Docker agents for isolated testing:

```groovy
pipeline {
    agent none

    stages {
        stage('Test in Docker') {
            matrix {
                axes {
                    axis {
                        name 'DOCKER_IMAGE'
                        values 'rust:latest', 'rust:1.70'
                    }
                }
                stages {
                    stage('Test') {
                        agent {
                            docker {
                                image "${DOCKER_IMAGE}"
                                args '--network host'
                            }
                        }
                        steps {
                            sh '''
                                cargo install --path .
                                clnrm pull tests/
                                clnrm run tests/ --parallel --workers 2
                            '''
                        }
                        post {
                            always {
                                junit 'test-results.xml'
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Azure DevOps Integration

### Azure DevOps Pipeline

Azure DevOps pipeline configuration:

```yaml
# azure-pipelines.yml
trigger:
  branches:
    include:
    - main
    - develop
  paths:
    exclude:
    - README.md
    - docs/

pool:
  vmImage: 'ubuntu-latest'

variables:
  rustVersion: '1.70'
  clnrmWorkers: '4'

steps:
- task: UseRust@0
  inputs:
    version: $(rustVersion)

- script: |
    cargo install --path .
    clnrm pull tests/
  displayName: 'Install clnrm and pull images'

- script: |
    clnrm run tests/unit/ --parallel --workers $(clnrmWorkers)
  displayName: 'Run unit tests'

- script: |
    docker run -d --name postgres -e POSTGRES_PASSWORD=testpass -e POSTGRES_DB=testdb -p 5432:5432 postgres:15-alpine
    docker run -d --name redis -p 6379:6379 redis:7-alpine
    sleep 10
    clnrm run tests/integration/ --external-services
  displayName: 'Run integration tests'
  condition: succeeded()

- script: |
    docker stop postgres redis
    docker rm postgres redis
  displayName: 'Cleanup containers'
  condition: always()

- script: |
    clnrm run tests/performance/ --baseline production
  displayName: 'Run performance tests'
  condition: and(succeeded(), eq(variables['Build.SourceBranch'], 'refs/heads/main'))

- task: PublishTestResults@2
  inputs:
    testResultsFormat: 'JUnit'
    testResultsFiles: 'test-results.xml'
  condition: always()

- task: PublishBuildArtifacts@1
  inputs:
    pathToPublish: '$(System.DefaultWorkingDirectory)'
    artifactName: 'test-artifacts'
    artifactType: 'Container'
  condition: always()
```

## CircleCI Integration

### CircleCI Configuration

CircleCI pipeline for clnrm:

```yaml
# .circleci/config.yml
version: 2.1

orbs:
  rust: circleci/rust@1.6.0

executors:
  test-executor:
    docker:
      - image: cimg/rust:1.70
    working_directory: ~/repo

  performance-executor:
    docker:
      - image: cimg/rust:1.70
    resource_class: large

jobs:
  install-dependencies:
    executor: test-executor
    steps:
      - checkout
      - rust/install
      - run:
          name: Install clnrm
          command: cargo install --path .
      - persist_to_workspace:
          root: ~/repo
          paths:
            - ~/.cargo/bin/clnrm

  test-unit:
    executor: test-executor
    steps:
      - checkout
      - attach_workspace:
          at: ~/repo
      - run:
          name: Run unit tests
          command: clnrm run tests/unit/ --parallel

  test-integration:
    executor: test-executor
    docker:
      - image: cimg/rust:1.70
      - image: postgres:15-alpine
        environment:
          POSTGRES_PASSWORD: testpass
          POSTGRES_DB: testdb
      - image: redis:7-alpine
    steps:
      - checkout
      - attach_workspace:
          at: ~/repo
      - run:
          name: Run integration tests
          command: clnrm run tests/integration/ --external-services

  performance-test:
    executor: performance-executor
    steps:
      - checkout
      - attach_workspace:
          at: ~/repo
      - run:
          name: Run performance tests
          command: clnrm run tests/performance/ --baseline production

workflows:
  version: 2
  test-workflow:
    jobs:
      - install-dependencies
      - test-unit:
          requires:
            - install-dependencies
      - test-integration:
          requires:
            - install-dependencies
      - performance-test:
          requires:
            - test-unit
          filters:
            branches:
              only: main
```

## Best Practices

### 1. Cache Dependencies

```yaml
# ✅ Good: Cache dependencies for faster builds
cache:
  paths:
    - ~/.cargo/registry
    - ~/.cargo/git
    - target/

- name: Install clnrm
  run: cargo install --path .
```

### 2. Use Parallel Execution

```bash
# ✅ Good: Parallel execution for faster tests
clnrm run tests/ --parallel --workers $(nproc)
```

### 3. Pull Images Early

```bash
# ✅ Good: Pull images early to avoid timeouts
- name: Pull test images
  run: clnrm pull tests/
```

### 4. Handle External Services

```yaml
# ✅ Good: Handle external services properly
services:
  postgres:
    image: postgres:15-alpine
    env:
      POSTGRES_PASSWORD: testpass

steps:
  - run: clnrm run tests/ --external-services
```

### 5. Use Artifacts for Results

```yaml
# ✅ Good: Upload test results as artifacts
- uses: actions/upload-artifact@v3
  if: always()
  with:
    name: test-results
    path: |
      *.json
      *.xml
      *.sha256
```

## Next Steps

Now that you understand CI/CD integration:

1. **Set up your pipeline**: Choose GitHub Actions, GitLab CI, or Jenkins for your project
2. **Configure parallel execution**: Set up parallel test execution for faster builds
3. **Learn performance tuning**: Move on to [Performance Tuning](performance-tuning.md)
4. **Implement enterprise patterns**: Learn about [Enterprise Patterns](enterprise-patterns.md)

## Further Reading

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitLab CI Documentation](https://docs.gitlab.com/ee/ci/)
- [Jenkins Pipeline Documentation](https://www.jenkins.io/doc/book/pipeline/)
- [Azure DevOps Documentation](https://docs.microsoft.com/en-us/azure/devops/)
