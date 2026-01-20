data "aws_region" "current" {}

output "dsql_endpoint" {
  value = "${aws_dsql_cluster.default.identifier}.dsql.${data.aws_region.current.region}.on.aws"
}