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
        //def appImage = "dwhyte40/vhennus_server:${env.BUILD_NUMBER}"
         sh 'docker build -t vhennus_server:latest .'
      }
    }
//     stage('Test') {
//       steps {
//         sh 'cargo test'
//       }
//     }
  }
}
