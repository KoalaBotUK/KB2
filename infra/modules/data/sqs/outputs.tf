output "queue_url" {
  value = aws_sqs_queue.default.url
}

output "queue_arn" {
  value = aws_sqs_queue.default.arn
}
