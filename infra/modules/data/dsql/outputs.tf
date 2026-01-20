data "aws_region" "current" {}

output "dsql_arn" {
  value = aws_dsql_cluster.default.arn
}

output "dsql_endpoint" {
  value = "${aws_dsql_cluster.default.identifier}.dsql.${data.aws_region.current.region}.on.aws"
}