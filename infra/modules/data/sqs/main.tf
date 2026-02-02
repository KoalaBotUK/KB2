resource "aws_sqs_queue" "dlq" {
  name                      = "kb2-queue-dlq-${var.deployment_env}"
}

resource "aws_sqs_queue" "default" {
  name                      = "kb2-queue-${var.deployment_env}"
  receive_wait_time_seconds = 10
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 4
  })
}

resource "aws_sqs_queue_redrive_allow_policy" "queue_redrive_allow_policy" {
  queue_url = aws_sqs_queue.dlq.id

  redrive_allow_policy = jsonencode({
    redrivePermission = "byQueue",
    sourceQueueArns   = [aws_sqs_queue.default.arn]
  })
}