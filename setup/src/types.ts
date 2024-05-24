import {RemoteSchemaDef} from "@hasura/metadata-api";
import {ActionDefinition, CustomTypes} from "@hasura/metadata";

export interface Action<T extends string, D> {
  "type": T,
  "args": {
    "name": string,
    "definition": D
  },
  "comment": string,
}

export type AddRemoteSchema = Action<"add_remote_schema", RemoteSchemaDef>;

export type AddAction = Action<"create_action", ActionDefinition>;
export type SetCustomTypes = {
  "type": "set_custom_types",
  "args": CustomTypes,
  "comment": string,
}


export interface Change {
  apply(): Promise<void>

  rollback(): Promise<void>
}

export type Actions = AddRemoteSchema | AddAction | SetCustomTypes

export type Bulk = {
  "type": "bulk",
  "source": "default",
  "args": Actions[]
}