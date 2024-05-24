import {CustomTypes} from '@hasura/metadata'
import {createGraphQLSchema, Oas2, Oas3} from 'openapi-to-graphql'
import {AddAction, Change, SetCustomTypes} from "./types.js";
import axios from "axios";
import {generateAction, generatedActionToHasuraAction} from "./hasura/utils.js"
import {getAllActionsFromSdl, getAllTypesFromSdl} from "./hasura/add/sdlUtils.js";
import {GeneratedAction, Operation, ResponseTranform} from "./hasura/types.js";
import {uniqBy} from "lodash"

const EMPTY_TYPES: CustomTypes = {
  scalars: [],
  enums: [],
  input_objects: [],
  objects: [],
}
const petstore = 'https://petstore3.swagger.io/api/v3/openapi.json'
const paypal =
  "https://raw.githubusercontent.com/paypal/paypal-rest-api-specifications/main/openapi/invoicing_v2.json"
const config = {
  headers: {
    "Content-Type": "application/json",
    "X-Hasura-Role": "admin",
    "X-Hasura-Admin-Secret": "admin_secret"
  }
}

async function apply() {
  const res = await axios.get(paypal)
  // const res = await axios.get(petstore)
  await parse(res.data)
}

const options = {
  fillEmptyResponses: true,
  operationIdFieldNames: true,
  simpleEnumValues: true,
  viewer: false,
  oasValidatorOptions: {
    warnOnly: true,
  },
  softValidation: true,
  report: {
    validationErrors: [],
    warnings: [],
    numOps: 0,
    numOpsQuery: 0,
    numOpsMutation: 0,
    numOpsSubscription: 0,
    numQueriesCreated: 0,
    numMutationsCreated: 0,
    numSubscriptionsCreated: 0,
  },
}

// Some of the parameters are put in the URL
// the parameters is start with `$body.input.`, wrap with {{}}, and the name is wrong, need to mapping by saneMap
// example: {{$base_url}}/v2/invoicing/invoices/{{$body.input.invoice_id}}/send
const CAPTURE_PARAMETER_NAME_REGEX = /\{\{\$body\.input\.([^}]+)\}\}/g

async function parse(oas: Oas3 | Oas2) {
  const result = await createGraphQLSchema(oas, options);
  const saneMapReverse = new Map(
    Object.entries(result.data.saneMap).map(([k, v]) => [v, k])
  )


  const parseResults = await Promise.all(Object.entries(result.data.operations)
    .map(async ([name, operation]) => {
      return await parseOperation(oas, operation)
    }))

  const makePayload = parseResults.map(ga => {
    const actions = getAllActionsFromSdl(ga.action)
    const translatedActionName = actions[0].name
    const hasuraAction = generatedActionToHasuraAction(ga)
    const matches = hasuraAction.requestTransform.url.matchAll(CAPTURE_PARAMETER_NAME_REGEX)
    for (const match of matches) {
      const paramName = match[1]
      const mappedName = saneMapReverse.get(paramName)
      if (mappedName) {
        hasuraAction.requestTransform.url = hasuraAction.requestTransform.url.replace(match[0], `{{$body.input.${mappedName}}}`)
      }
    }

    // TODO: remove this dirty hack
    addStaticMessageIntoResponseTransform(hasuraAction.responseTransform)

    const mergedActionDef = {
      name: translatedActionName,
      definition: {
        ...actions[0].definition,
        forward_client_headers: true,
        handler: ga.baseUrl,
        headers: ga.headers,
        kind: "synchronous",
        request_transform: hasuraAction.requestTransform,
        response_transform: hasuraAction.responseTransform,
      },
    };

    let customTypes: CustomTypes = EMPTY_TYPES;
    if (ga.types.trim().length !== 0) {
      customTypes = getAllTypesFromSdl(ga.types)
    }

    return {
      customTypes,
      actionDefinition: mergedActionDef,
    }
  });

  try {
    const collectedTypes = collectCustomTypes(
      makePayload.map(p => p.customTypes)
    )

    const setTypePayload: SetCustomTypes = {
      type: "set_custom_types",
      args: collectedTypes,
      comment: "Add custom types"
    }

    const setCustomTypesRes = await axios.post('http://localhost:8080/v1/metadata', setTypePayload, config)
    console.log(setCustomTypesRes.data)

  } catch (e) {
    console.error(e.response.data)
  }
  for (const payload of makePayload) {
    const addActionPayload: AddAction = {
      type: "create_action",
      args: payload.actionDefinition,
      comment: "Add action"
    }

    try {
      const addActionRes = await axios.post('http://localhost:8080/v1/metadata', addActionPayload, config)
      console.log(addActionRes.data)
    } catch (e) {
      console.error(e.response.data)
    }
  }
}

async function parseOperation(oas: Oas2 | Oas3, op: Operation): Promise<GeneratedAction> {
  return await generateAction(oas, op.operationId)
}

function collectCustomTypes(types: CustomTypes[]): CustomTypes {
  const collectedTypes = types
    .reduce((acc, curr) => {
      return {
        scalars: [...acc.scalars, ...curr.scalars],
        enums: [...acc.enums, ...curr.enums],
        input_objects: [...acc.input_objects, ...curr.input_objects],
        objects: [...acc.objects, ...curr.objects],
      }
    }, EMPTY_TYPES)

  return {
    scalars: uniqBy(collectedTypes.scalars, 'name'),
    enums: uniqBy(collectedTypes.enums, 'name'),
    input_objects: uniqBy(collectedTypes.input_objects, 'name'),
    objects: uniqBy(collectedTypes.objects, 'name'),
  }

}

// TODO: remove this dirty hack
// https://github.com/hasura/graphql-engine/issues/9880
// Since there are some bug in hasura engine, as a workaround, just add a static message into response transform
// then the response will work when the response code is 4xx
function addStaticMessageIntoResponseTransform(rt?: ResponseTranform) {
  const staticMessage = `
    "message": "This is a dirty hack, please remove this if https://github.com/hasura/graphql-engine/issues/9880 has fixed",
  `

  if (rt?.body?.template?.startsWith('{\n')) {
    // replace head '{' with '{' + staticMessage
    rt.body.template = rt.body.template.replace('{\n', '{' + staticMessage + '\n')
  }
}

async function rollback() {
}

const change: Change = {
  apply,
  rollback
}

export default change