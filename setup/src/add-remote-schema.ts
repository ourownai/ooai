import {RemoteSchemaDef} from '@hasura/metadata-api'
import axios from 'axios';
import {AddRemoteSchema, Change} from './types.js';

const remoteSchema: RemoteSchemaDef = {
  "url": "http://host.docker.internal:8000",
  "headers": [{"name": "X-Server-Request-From", "value": "Hasura"}],
  "forward_client_headers": false,
  "timeout_seconds": 60,
  "customization": {
    "root_fields_namespace": "some_field_name",
    "type_names": {
      "prefix": "some_type_name_prefix",
      "suffix": "some_type_name_suffix",
      "mapping": {
        "some_type_name": "some_new_type_name"
      }
    },
    "field_names": [{
      "parent_type": "some_type_name",
      "prefix": "some_field_name_prefix",
      "suffix": "some_field_name_suffix",
      "mapping": {
        "some_field_name": "some_new_field_name"
      }
    }]
  }
};

// POST /v1/metadata HTTP/1.1
// Content-Type: application/json
// X-Hasura-Role: admin
async function apply() {
  const action: AddRemoteSchema = {
    "type": "add_remote_schema",
    "args": {
      "name": "data-exchange",
      "definition": remoteSchema
    },
    "comment": "Add remote schema"
  }

  const resp = await axios.post('http://localhost:8080/v1/metadata', action, {
    headers: {
      "Content-Type": "application/json",
      "X-Hasura-Role": "admin",
      "X-Hasura-Admin-Secret": "admin_secret"
    }
  })

  console.log(resp.status)
}

async function rollback() {
  // TODO: rollback
}

const change: Change = {
  apply,
  rollback
}

export default change