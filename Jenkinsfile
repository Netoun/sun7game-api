pipeline {
  agent any
  stages {
    stage('build') {
      steps {
        build 'cargo build --release'
      }
    }
  }
}