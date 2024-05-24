/*
 * Original file path: https://github.com/hasura/graphql-engine/blob/master/frontend/libs/console/legacy-ce/src/lib/shared/utils/hasuraCustomTypeUtils.js
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

import {unwrapType} from './wrappingTypeUtils.js';

export const inbuiltTypes = {
  Int: true,
  Boolean: true,
  String: true,
  Float: true,
  ID: true,
};

const singularize = kind => {
  return kind.substr(0, kind.length - 1);
};

export const filterNameLessTypeLess = arr => {
  return arr.filter(item => !!item.name && !!item.type);
};

export const filterNameless = arr => {
  return arr.filter(item => !!item.name);
};

export const filterValueLess = arr => {
  return arr.filter(item => !!item.value);
};

export const mergeCustomTypes = (newTypesList, existingTypesList) => {
  const mergedTypes = [...existingTypesList];
  const overlappingTypenames = [];
  const existingTypeIndexMap = {};

  existingTypesList.forEach((et, i) => {
    existingTypeIndexMap[et.name] = i;
  });

  newTypesList.forEach(nt => {
    if (existingTypeIndexMap[nt.name] !== undefined) {
      mergedTypes[existingTypeIndexMap[nt.name]] = nt;
      overlappingTypenames.push(nt.name);
    } else {
      mergedTypes.push(nt);
    }
  });

  return {
    types: mergedTypes,
    overlappingTypenames,
  };
};

export const reformCustomTypes = typesFromState => {
  const sanitisedTypes = [];
  typesFromState.forEach(t => {
    if (!t.name) {
      return;
    }
    const sanitisedType = {...t};
    if (t.fields) {
      sanitisedType.fields = filterNameLessTypeLess(t.fields);
    }
    if (t.arguments) {
      sanitisedType.arguments = filterNameLessTypeLess(t.arguments);
    }

    sanitisedTypes.push(sanitisedType);
  });

  const customTypes = {
    scalars: [],
    input_objects: [],
    objects: [],
    enums: [],
  };

  sanitisedTypes.forEach(_type => {
    const type = JSON.parse(JSON.stringify(_type));
    delete type.kind;
    switch (_type.kind) {
      case 'scalar':
        customTypes.scalars.push(type);
        return;
      case 'object':
        customTypes.objects.push(type);
        return;
      case 'input_object':
        customTypes.input_objects.push(type);
        return;
      case 'enum':
        customTypes.enums.push(type);
        return;
      default:
        return;
    }
  });

  return customTypes;
};

export const parseCustomTypes = customTypesServer => {
  const customTypesClient = [];
  Object.keys(customTypesServer).forEach(tk => {
    const types = customTypesServer[tk];
    if (types) {
      types.forEach(t => {
        customTypesClient.push({
          ...t,
          kind: singularize(tk),
        });
      });
    }
  });
  return customTypesClient;
};

export const getActionTypes = (actionDef, allTypes) => {
  const usedTypes = {};
  const actionTypes = [];

  const getDependentTypes = typename => {
    if (usedTypes[typename]) return;
    const type = allTypes.find(t => t.name === typename);
    if (!type) return;
    actionTypes.push(type);
    usedTypes[typename] = true;
    if (type.kind === 'input_object' || type.kind === 'object') {
      type.fields.forEach(f => {
        const {typename: _typename} = unwrapType(f.type);
        getDependentTypes(_typename);
      });
    }
  };

  actionDef.arguments.forEach(a => {
    const {typename} = unwrapType(a.type);
    getDependentTypes(typename);
  });

  getDependentTypes(actionDef.output_type);

  return actionTypes;
};

export const hydrateTypeRelationships = (newTypes, existingTypes) => {
  const typeMap = {};
  existingTypes.forEach(t => {
    typeMap[t.name] = t;
  });

  return newTypes.map(t => {
    if (t.kind === 'object' && typeMap[t.name]) {
      return {
        ...t,
        ...(typeMap[t.name].relationships && {
          relationships: typeMap[t.name].relationships,
        }),
      };
    }

    return t;
  });
};
