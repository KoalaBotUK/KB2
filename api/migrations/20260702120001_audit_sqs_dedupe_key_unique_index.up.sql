-- no-transaction
CREATE UNIQUE INDEX ASYNC audit_sqs_message_id_key ON audit (sqs_message_id);
