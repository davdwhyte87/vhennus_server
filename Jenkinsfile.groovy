pipeline {
  agent any
  stages {
    stage('Checkout') {
      steps {
        checkout scm
      }
    }
    stage('Build') {
      steps {
        def appImage = "dwhyte40/vhennus_server:${env.BUILD_NUMBER}"
      }
    }
    stage('Test') {
      steps {
        sh 'cargo test'
      }
    }
  }
}
