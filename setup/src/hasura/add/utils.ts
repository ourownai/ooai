/*
 * Original file path: https://github.com/hasura/graphql-engine/blob/master/frontend/libs/console/legacy-ce/src/lib/components/Services/Actions/Add/utils.ts#L18
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

import {getActionDefinitionFromSdl, getTypesFromSdl,} from './sdlUtils.js';


export type Nullable<T> = T | null | undefined;

type ArgType = { name: string; type: string; description: string };

// Removes ! from type, and returns a new string
const getTrimmedType = (value: string): string => {
  const typeName =
    value[value.length - 1] === '!'
      ? value.substring(0, value.length - 1)
      : value;
  return typeName;
};

const getArgObjFromDefinition = (
  arg: ArgType,
  typesdef: Record<string, any>
): Nullable<Record<string, any>> => {
  let type = arg?.type;
  type = getTrimmedType(type);
  const name = arg?.name;
  if (type === 'String' || type === 'ID') return {[name]: `${name}`};
  if (type === 'Int' || type === 'Float' || type === 'BigInt')
    return {[name]: 10};
  if (type === 'Boolean') return {[name]: false};
  if (type === '[String]' || type === '[ID]') {
    return {[name]: ['foo', 'bar']};
  }
  if (type === '[Int]' || type === '[Float]' || type === '[BigInt]') {
    return {[name]: [10, 20]};
  }

  const userDefType = typesdef?.types.find(
    (t: Record<string, any>) => t.name === type
  );
  if (userDefType?.kind === 'input_object') {
    let obj = {};
    userDefType?.fields?.forEach((f: ArgType) => {
      obj = {...obj, ...getArgObjFromDefinition(f, typesdef)};
    });
    return {
      [name]: obj,
    };
  }

  if (userDefType?.kind === 'enum') {
    return {
      [name]: userDefType.values?.[0]?.value ?? '',
    };
  }

  return {};
};

export const getActionRequestSampleInput = (
  actionSdl: string,
  typesSdl: string
): string => {
  const actionDef = getActionDefinitionFromSdl(actionSdl);
  const typesDef = getTypesFromSdl(typesSdl);
  let inputObj = {};

  // pass all top level args
  actionDef?.arguments?.forEach((arg: ArgType) => {
    inputObj = {...inputObj, ...getArgObjFromDefinition(arg, typesDef)};
  });

  const obj = {
    action: {
      name: actionDef?.name,
    },
    input: {
      ...inputObj,
    },
  };

  const value = JSON.stringify(obj, null, 2);
  return value;
};
