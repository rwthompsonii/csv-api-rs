this project is just a quick and dirty showing how to do serialization/deserialization for CSV to JSON

it hosts a server listening at 12345.

don't use it for anything serious

it needs a local POSTGRES server at localhost to function, with a username of "postgres" and a password of "password", and it expects that user to have the ability to create tables, ie, the "CreateDB" permission.

provided is a test csv file (`csv_for_test`) that can be POST'ed at the /csv endpoint.
example: ```curl localhost:12345/csv --data-binary @/full/path/to/csv_for_test```

after the POST writes the rows, individual records can be queried via GET on the /csv/<record_id> path:
example: ```curl -vvv localhost:12345/csv/yet-another-string```

Thanks for playing.
