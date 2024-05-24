/*
 * Original file path: https://github.com/hasura/graphql-engine/blob/master/frontend/libs/console/legacy-ce/src/lib/shared/utils/wrappingTypeUtils.ts
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

import {GraphQLInputObjectType, GraphQLObjectType, isListType, isNonNullType, isWrappingType, TypeNode,} from 'graphql';

const unwrapNonNullable = (wrappedTypename: string) => {
  return wrappedTypename.substring(0, wrappedTypename.length - 1);
};

const unwrapList = (wrappedTypename: string) => {
  return wrappedTypename.substring(1, wrappedTypename.length - 1);
};

export const unwrapType = (wrappedTypename: string) => {
  let typename = wrappedTypename;
  const typeWrapperStack = [];
  let lastChar = typename.charAt(typename.length - 1);

  while (lastChar) {
    if (lastChar === ']') {
      typename = unwrapList(typename);
      typeWrapperStack.push('l');
    } else if (lastChar === '!') {
      typename = unwrapNonNullable(typename);
      typeWrapperStack.push('n');
    } else {
      break;
    }
    lastChar = typename.charAt(typename.length - 1);
  }

  return {
    stack: typeWrapperStack,
    typename,
  };
};

export const getAstTypeMetadata = (type: TypeNode) => {
  let node = {...type};
  const typewraps = [];
  while (node.kind !== 'NamedType') {
    if (node.kind === 'ListType') {
      typewraps.push('l');
    }
    if (node.kind === 'NonNullType') {
      typewraps.push('n');
    }
    node = node.type;
  }
  const typename = node.name.value;
  return {
    typename,
    stack: typewraps,
  };
};

export const getSchemaTypeMetadata = (
  type: GraphQLObjectType | GraphQLInputObjectType
) => {
  let t = type;
  const typewraps = [];
  while (isWrappingType(t)) {
    if (isListType(t)) {
      typewraps.push('l');
    }
    if (isNonNullType(t)) {
      typewraps.push('n');
    }
    // @ts-ignore
    t = t.ofType;
  }

  return {
    typename: t.name,
    stack: typewraps,
  };
};

export const wrapTypename = (name: string, wrapperStack: string[]) => {
  let wrappedTypename = name;
  wrapperStack.reverse().forEach(w => {
    if (w === 'l') {
      wrappedTypename = `[${wrappedTypename}]`;
    }
    if (w === 'n') {
      wrappedTypename = `${wrappedTypename}!`;
    }
  });
  return wrappedTypename;
};
