pipeline {
  agent any
  stages {
    stage('First Step') {
      steps {
        echo 'Run CI Project'
        sh 'git pull origin master'
      }
    }
  }
}