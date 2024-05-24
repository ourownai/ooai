import {ActionDefinition, CustomTypes} from '@hasura/metadata'
import {AddAction, Change} from "./types.js";
import axios from "axios";

const customTypes: CustomTypes = {
  "scalars": [],
  "enums": [],
  "input_objects": [],
  "objects": [
    {
      "name": "LoginResponse",
      "fields": [
        {
          "name": "accessToken",
          "type": "String!"
        }
      ]
    }
  ]
}

const actionDef: ActionDefinition = {
  "kind": "synchronous",
  // @ts-ignore
  "type": "mutation",
  "arguments": [
    {
      "name": "username",
      "type": "String!"
    },
    {
      "name": "password",
      "type": "String!"
    }
  ],
  "output_type": "LoginResponse",
  "handler": "http://host.docker.internal:8000/create-user",
}

const config = {
  headers: {
    "Content-Type": "application/json",
    "X-Hasura-Role": "admin",
    "X-Hasura-Admin-Secret": "admin_secret"
  }
}

async function apply() {

  const set_custom_types = {
    "type": "set_custom_types",
    "args": customTypes,
    "comment": "Add custom types"
  }

  const resp_custom_type = await axios.post('http://localhost:8080/v1/metadata', set_custom_types, config)

  console.log(resp_custom_type.status)

  const add: AddAction = {
    "type": "create_action",
    "args": {
      "name": "data_exchange_action",
      "definition": actionDef,
    },
    "comment": "Custom action to create user"
  }

  const resp = await axios.post('http://localhost:8080/v1/metadata', add, config)

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