import {EditorView, basicSetup} from "codemirror"
import {EditorState } from "@codemirror/state"
import { jsonSchema } from "codemirror-json-schema";
import requestJsonSchema from "./request.schema.json";
import {diagnosticCount} from "@codemirror/lint"
import { jsonLanguage } from "@codemirror/lang-json";

const schema = requestJsonSchema
const validEvent = new CustomEvent("validityChange", {detail:{valid: true}})
const invalidEvent = new CustomEvent("validityChange", {detail:{valid: false}})
const exampleJson = `{
  "first_name": "Bob",
  "last_name": "Bobson",
  "address": {
    "lines": ["123 Bob Lane"],
    "postcode": "BO11OB"
  }
}`

let lastValidJson = null
let inputElement = null
let editor = null;
let viewer = null;

function handle_docchange(state) {
  if (diagnosticCount(state) > 0) {
    if (lastValidJson !== false) {
      inputElement.dispatchEvent(invalidEvent)
      lastValidJson = false
    }
  } else {
    if (lastValidJson !== true) {
      inputElement.dispatchEvent(validEvent)
      lastValidJson = true
    }
  }
}

function input_state(doc=``) {
  let state = EditorState.create({
    doc,
    extensions: [
      basicSetup,
      jsonSchema(schema),
      EditorView.updateListener.of((update) => handle_docchange(update.state))
    ]
  })
  handle_docchange(state)
  return state
}

function output_state(doc=``) {
  return EditorState.create({
    doc,
    extensions: [
      basicSetup,
      jsonLanguage,
      EditorState.readOnly.of(true)
    ]
  })
}


function send() {
  viewer.setState(output_state())
  let req_body = editor.state.doc.toString()
  console.log(req_body)
  fetch("http://localhost:8080/api/query", {
      method: "POST",
      body: req_body,
      headers: {
        "Content-type": "application/json; charset=UTF-8"
      }
    })
    .then((response) => response.json())
    .then((json) => {
      viewer.setState(output_state(JSON.stringify(json, undefined, 2)))
    })
}

function setup(input, output) {
  inputElement = input
  editor = new EditorView({
    state: input_state(exampleJson),
    parent: input
  })

  viewer = new EditorView({
    state: output_state(),
    parent: output
  })
}

export default {
  send,
  setup
}
