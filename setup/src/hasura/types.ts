/*
 * Original file path: https://github.com/hasura/graphql-engine/blob/master/frontend/libs/console/legacy-ce/src/lib/features/Actions/components/OASGenerator/types.ts
 */

// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing,
//   software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
//   specific language governing permissions and limitations
// under the License.

const {createGraphQLSchema} = require('openapi-to-graphql')

export type Nullable<T> = T | null;

export type ReferenceObject = {
  $ref: string;
};

type SchemaObjectType =
  | 'string'
  | 'number'
  | 'integer'
  | 'boolean'
  | 'object'
  | 'array';

export type SchemaObject = {
  title?: string;
  type?: SchemaObjectType | [SchemaObjectType, null];
  format?: string;
  nullable?: boolean;
  description?: string;
  properties?: {
    [key: string]: SchemaObject | ReferenceObject;
  };
  required?: string[];
  default?: any;
  additionalProperties?: SchemaObject | ReferenceObject | boolean;
  items?: SchemaObject | ReferenceObject; // MUST be a single schema object in OAS, see https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#properties
  additionalItems?: boolean | string[];
  enum?: string[];
  allOf?: (SchemaObject | ReferenceObject)[];
  anyOf?: (SchemaObject | ReferenceObject)[];
  oneOf?: (SchemaObject | ReferenceObject)[];
  not?: (SchemaObject | ReferenceObject)[];
};

export type RequestTransformForGeneratedAction =
  | {
  type: 'json';
  value: string;
}
  | {
  type: 'x-www-form-urlencoded';
  value: Record<string, string>;
};

export type GeneratedAction = {
  operationId: string;
  actionType: 'query' | 'mutation';
  types: string;
  action: string;
  description: string;
  method: RequestTransformMethod;
  baseUrl: string;
  path: string;
  requestTransforms?: RequestTransformForGeneratedAction;
  responseTransforms: string;
  sampleInput: string;
  headers: string[];
  queryParams: string | { name: string; value: string }[];
};


/**
 * https://hasura.io/docs/latest/graphql/core/api-reference/syntax-defs.html#requesttransformation
 */
export type RequestTransformMethod =
  | 'POST'
  | 'GET'
  | 'PUT'
  | 'DELETE'
  | 'PATCH';

export type RequestTransformContentType =
  | 'application/json'
  | 'application/x-www-form-urlencoded';

export type RequestTransformBodyActions =
  | 'remove'
  | 'transform'
  | 'x_www_form_urlencoded';

export type RequestTransformBody = {
  action: RequestTransformBodyActions;
  template?: string;
  form_template?: Record<string, string> | string;
};

export type ResponseTransformBody = {
  action: RequestTransformBodyActions;
  template?: string;
};

export type RequestTransformHeaders = {
  add_headers?: Record<string, string>;
  remove_headers?: string[];
};

export type RequestTransformTemplateEngine = 'Kriti';

interface RequestTransformFields {
  method?: Nullable<RequestTransformMethod>;
  url?: Nullable<string>;
  content_type?: Nullable<RequestTransformContentType>;
  request_headers?: Nullable<RequestTransformHeaders>;
  query_params?: Nullable<Record<string, string>> | string;
  template_engine?: Nullable<RequestTransformTemplateEngine>;
}

/**
 * https://hasura.io/docs/latest/graphql/core/api-reference/syntax-defs/#requesttransformation
 */
interface RequestTransformV1 extends RequestTransformFields {
  version: 1;
  body?: string;
}

interface RequestTransformV2 extends RequestTransformFields {
  version: 2;
  body?: RequestTransformBody;
}

export type RequestTransform = RequestTransformV1 | RequestTransformV2;

export type ResponseTranform = {
  version: 2;
  body?: ResponseTransformBody;
  template_engine?: Nullable<RequestTransformTemplateEngine>;
};

export type Result = Awaited<ReturnType<typeof createGraphQLSchema>>;
export type Operation = Result['data']['operations'][0];

export type DataDefinition = Operation['responseDefinition'];
export type SubDefinition = Operation['responseDefinition']['subDefinitions'];

export type OperationParameters = Operation['parameters'];

export interface OASError extends Error {
  message: string;
  options: {
    context: string[];
  };
}