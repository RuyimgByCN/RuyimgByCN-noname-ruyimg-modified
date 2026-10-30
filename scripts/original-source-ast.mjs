import { readFile } from "node:fs/promises";
import ts from "../apps/web-client/node_modules/typescript/lib/typescript.js";

export async function parseObjectFile(file, variableName, allowDiagnostics = false) {
  const sourceFile = await parseJavaScript(file, allowDiagnostics);
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (
        declaration.name.getText(sourceFile) === variableName &&
        ts.isObjectLiteralExpression(declaration.initializer)
      ) {
        return { sourceFile, properties: declaration.initializer.properties };
      }
    }
  }
  throw new Error(`${file}: const ${variableName} = { ... } not found`);
}

export async function parseJavaScript(file, allowDiagnostics = false) {
  const source = await readFile(file, "utf8");
  const sourceFile = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
  if (!allowDiagnostics && sourceFile.parseDiagnostics.length) {
    throw new Error(`${file}: TypeScript parser reported ${sourceFile.parseDiagnostics.length} errors`);
  }
  return sourceFile;
}

export function safeLiteral(node, location = "$") {
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
    return { ok: true, value: node.text, omitted: [] };
  }
  if (ts.isNumericLiteral(node)) {
    const value = Number(node.text);
    return Number.isFinite(value)
      ? { ok: true, value, omitted: [] }
      : { ok: false, omitted: [location] };
  }
  if (node.kind === ts.SyntaxKind.TrueKeyword) {
    return { ok: true, value: true, omitted: [] };
  }
  if (node.kind === ts.SyntaxKind.FalseKeyword) {
    return { ok: true, value: false, omitted: [] };
  }
  if (node.kind === ts.SyntaxKind.NullKeyword) {
    return { ok: true, value: null, omitted: [] };
  }
  if (
    ts.isPrefixUnaryExpression(node) &&
    ts.isNumericLiteral(node.operand) &&
    [ts.SyntaxKind.PlusToken, ts.SyntaxKind.MinusToken].includes(node.operator)
  ) {
    const unsigned = Number(node.operand.text);
    const value = node.operator === ts.SyntaxKind.MinusToken ? -unsigned : unsigned;
    return Number.isFinite(value)
      ? { ok: true, value, omitted: [] }
      : { ok: false, omitted: [location] };
  }
  if (ts.isArrayLiteralExpression(node)) {
    const value = [];
    const omitted = [];
    for (const [index, element] of node.elements.entries()) {
      if (ts.isSpreadElement(element)) return { ok: false, omitted: [`${location}[${index}]`] };
      const parsed = safeLiteral(element, `${location}[${index}]`);
      if (!parsed.ok) return { ok: false, omitted: parsed.omitted };
      value.push(parsed.value);
      omitted.push(...parsed.omitted);
    }
    return { ok: true, value, omitted };
  }
  if (ts.isObjectLiteralExpression(node)) {
    const value = Object.create(null);
    const omitted = [];
    for (const property of node.properties) {
      const name = propertyName(property);
      if (!name || !ts.isPropertyAssignment(property) ||
        ["__proto__", "prototype", "constructor"].includes(name)) {
        omitted.push(`${location}.*`);
        continue;
      }
      const parsed = safeLiteral(property.initializer, `${location}.${name}`);
      if (parsed.ok) value[name] = parsed.value;
      omitted.push(...parsed.omitted);
    }
    return { ok: true, value, omitted };
  }
  return { ok: false, omitted: [location] };
}

export function propertyName(property) {
  const name = property.name;
  if (!name) return null;
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) return name.text;
  return null;
}


export async function parseDefaultObjectFile(file) {
  const sourceFile = await parseJavaScript(file);
  const statement = sourceFile.statements.find((node) => ts.isExportAssignment(node) && ts.isObjectLiteralExpression(node.expression));
  return statement ? { sourceFile, properties: statement.expression.properties } : null;
}
