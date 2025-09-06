// // import { parser } from "../grammar/grammar.js";
// // import { LRLanguage, LanguageSupport } from "@codemirror/language";
// // import { styleTags, tags as t } from "@codemirror/highlight";

// // const parserWithHighlight = parser.configure({
// //   props: [
// //     styleTags({
// //       "Keyword": t.keyword,
// //     })
// //   ]
// // });

// // export const quantaLang = LRLanguage.define({ parser: parserWithHighlight.configure({}) });
// // export const quanta = () => new LanguageSupport(quantaLang);

// import { parser } from "../grammar/grammar.js";
// import { LRLanguage, LanguageSupport } from '@codemirror/language';
// import {
//   autocompletion,
// } from '@codemirror/autocomplete';

// // Create a Lezer-based language extension
// // export const jsonLanguage = LRLanguage.define({
// //   name: "json",
// //   parser: parser
// // //   parser: parser.configure({
// // //     props: [
// // //       indentNodeProp.add({
// // //         Object: continuedIndent({except: /^\s*\}/}),
// // //         Array: continuedIndent({except: /^\s*\]/})
// // //       }),
// // //       foldNodeProp.add({
// // //         "Object Array": foldInside
// // //       })
// // //     ]
// // //   }),
// // //   languageData: {
// // //     closeBrackets: {brackets: ["[", "{", '"']},
// // //     indentOnInput: /^\s*[\}\]]$/
// // //   }
// // })

// // A simple completion source
// const myLanguageCompletions = (
//   context
// ) => {
//   const nodeBefore = myLanguage.parser.get('nodeBefore');
//   const tree = context.state.tree;
//   const pos = context.pos;
//   const completions = [];

//   // Get the syntax node at the current position
//   const node = tree.resolve(pos, -1);
//   console.log('Node at cursor:', node);

//   // Example: suggest keywords if the cursor is in an empty spot or at the start of an expression.
//   if (
//     node.name === 'Program' ||
//     (node.name === 'Expression' && node.from === pos)
//   ) {
//     completions.push({ label: 'let', type: 'keyword' });
//   }

//   // Example: suggest variables
//   if (node.name === 'Variable') {
//     // In a real scenario, you would look up available variables from the document's syntax tree
//     completions.push({ label: 'myVariable', type: 'variable' });
//     completions.push({ label: 'anotherVar', type: 'variable' });
//   }

//   // A more advanced approach would involve traversing the syntax tree to find contextual information.
//   // For instance, if you are inside a "Call" node, you might suggest function names.

//   // Return the results
//   return {
//     from: context.pos,
//     options: completions,
//   };
// };

// // Create the language support extension
// export const quanta = () => {};//new LanguageSupport(jsonLanguage);

import {parser} from "../grammar/grammar.js";
import {LRLanguage, LanguageSupport, indentNodeProp, foldNodeProp, foldInside, delimitedIndent} from "@codemirror/language"
import {styleTags, tags as t} from "@lezer/highlight"
import {completeFromList} from "@codemirror/autocomplete"

export const QuantaLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      indentNodeProp.add({
        Application: delimitedIndent({closing: ")", align: false})
      }),
      foldNodeProp.add({
        Application: foldInside
      }),
      styleTags({
        Identifier: t.variableName,
        Boolean: t.bool,
        String: t.string,
        LineComment: t.lineComment,
        "( )": t.paren
      })
    ]
  }),
  languageData: {
    commentTokens: {line: ";"}
  }
})

export const quantaCompletion = QuantaLanguage.data.of({
  autocomplete: completeFromList([
    {label: "bool", type: "keyword"},
    {label: "int", type: "keyword"},
    {label: "float", type: "keyword"},
    {label: "Color", type: "keyword"},
    {label: "Green", type: "keyword"},
    {label: "Blue", type: "keyword"},
    {label: "Red", type: "keyword"},
    {label: "Cyan", type: "keyword"},
    {label: "Pink", type: "keyword"},
    {label: "Black", type: "keyword"},
    {label: "White", type: "keyword"},
    {label: "circle", type: "function"},
    {label: "rectangle", type: "function"},
    {label: "line", type: "function"},
    {label: "setLineColor", type: "function"},
    {label: "setFigureColor", type: "function"},
  ])
})

export function quanta() {
  return new LanguageSupport(QuantaLanguage, [quantaCompletion])
}