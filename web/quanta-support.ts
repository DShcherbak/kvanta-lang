// // import { parser } from "../grammar/grammar.js";
// // import { LRLanguage, LanguageSupport } from "@codemirror/language";
// // import { styleTags, tags as t } from "@codemirror/highlight";

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
import {LRLanguage, LanguageSupport, indentNodeProp, foldNodeProp, foldInside, delimitedIndent, HighlightStyle, syntaxHighlighting} from "@codemirror/language"
import {highlightCode, highlightTree, styleTags, tags as t} from "@lezer/highlight"
import {completeFromList} from "@codemirror/autocomplete"
import { rustHighlighting } from "../grammar/highlight.js";

export const quantaHighlightStyle = HighlightStyle.define([
  // keywords
  { tag: t.keyword, color: "rgba(255, 0, 0, 1)" },
  {
    tag: t.operator,
    color: "rgba(255, 0, 0, 1)",
  },
  { tag: t.controlKeyword, color: "rgba(255, 255, 0, 1)" },
 // { tag: t.if, color: "#ff0000", fontWeight: "600" },
  // { tag: t.keyword, color: "var(--qt-keyword)" },
  // { tag: t.definitionKeyword, color: "var(--qt-keyword-def)", fontWeight: "600" },

  // // types & defs
  // { tag: t.typeName, color: "var(--qt-type)", fontStyle: "italic" },
  // { tag: t.definition(t.variableName), color: "var(--qt-def)", fontWeight: "600" },
  // { tag: t.variableName, color: "var(--qt-var)" },
  // { tag: t.function(t.variableName), color: "var(--qt-fn)" },
  // { tag: t.propertyName, color: "var(--qt-prop)" },

  // // literals
  // { tag: [t.string, t.special(t.string)], color: "var(--qt-str)" },
  // { tag: t.number, color: "var(--qt-num)" },
  // { tag: t.bool, color: "var(--qt-bool)", fontWeight: "600" },

  // // operators / punctuation / brackets
  // { tag: [t.operator, t.compareOperator, t.logicOperator, t.arithmeticOperator], color: "var(--qt-op)" },
  // { tag: [t.punctuation, t.separator], color: "var(--qt-punc)" },
  // { tag: [t.paren, t.brace, t.squareBracket], color: "var(--qt-bracket)" },

  // // comments & misc
  // { tag: t.comment, color: "var(--qt-comment)", fontStyle: "italic" },
  // { tag: t.invalid, color: "var(--qt-error)", textDecoration: "wavy underline" },
  // { tag: t.meta, color: "var(--qt-meta)" }
]);

export const quantaSyntax = syntaxHighlighting(quantaHighlightStyle);

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
        //Identifier: t.variableName,
        Boolean: t.bool,
        String: t.string,
        LineComment: t.lineComment,
        "( )": t.paren,
        "{ }": t.bracket
      })
    ]
  }),
  languageData: {
    commentTokens: {line: ";"},
  }
})






export const quantaCompletion = QuantaLanguage.data.of({
  autocomplete: completeFromList([
    {label: "bool", type: "keyword"},
    {label: "int", type: "keyword"},
    {label: "float", type: "keyword"},
    {label: "Color", type: "keyword"},
     {label: "Red", type: "keyword"},
  {label: "DarkRed", type: "keyword"},
  {label: "LightRed", type: "keyword"},

  {label: "Green", type: "keyword"},
  {label: "DarkGreen", type: "keyword"},
  {label: "LightGreen", type: "keyword"},

  {label: "Blue", type: "keyword"},
  {label: "DarkBlue", type: "keyword"},
  {label: "LightBlue", type: "keyword"},

  {label: "Yellow", type: "keyword"},
  {label: "DarkYellow", type: "keyword"},
  {label: "LightYellow", type: "keyword"},

  {label: "Orange", type: "keyword"},
  {label: "DarkOrange", type: "keyword"},
  {label: "LightOrange", type: "keyword"},

  {label: "Pink", type: "keyword"},
  {label: "LightPink", type: "keyword"},
  {label: "HotPink", type: "keyword"},

  {label: "Purple", type: "keyword"},
  {label: "Violet", type: "keyword"},
  {label: "DarkViolet", type: "keyword"},
  {label: "LightViolet", type: "keyword"},

  {label: "Brown", type: "keyword"},
  {label: "DarkBrown", type: "keyword"},
  {label: "LightBrown", type: "keyword"},

  {label: "Cyan", type: "keyword"},
  {label: "DarkCyan", type: "keyword"},
  {label: "LightCyan", type: "keyword"},

  {label: "Black", type: "keyword"},
  {label: "Gray", type: "keyword"},
  {label: "DarkGray", type: "keyword"},
  {label: "LightGray", type: "keyword"},
  {label: "White", type: "keyword"},
  {label: "Background", type: "keyword"},
  {label: "Transparent", type: "keyword"},
  {label: "Random", type: "keyword"},

    {label: "circle", type: "function"},
    {label: "rectangle", type: "function"},
    {label: "line", type: "function"},
    {label: "setLineColor", type: "function"},
    {label: "setFigureColor", type: "function"},
  ])
})

export function quanta() {
  return new LanguageSupport(QuantaLanguage, [quantaSyntax, quantaCompletion])
}

export const quantaLanguageSupport = new LanguageSupport(QuantaLanguage, [
  // Add the syntax highlighting extension, which uses your defined style
  quantaSyntax, quantaCompletion
]);