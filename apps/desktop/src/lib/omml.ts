import omml2mathmlLib from "omml2mathml";

/**
 * Extract plain text from OMML as fallback when conversion fails
 */
function extractTextFromOmml(omml: string): string {
  const textMatches = omml.match(/<m:t[^>]*>([^<]+)<\/m:t>/g);
  if (textMatches) {
    return textMatches
      .map(match => match.replace(/<m:t[^>]*>|<\/m:t>/g, ''))
      .join('');
  }
  return '';
}

export function ommlToMathml(omml: string): string {
  try {
    const fn = (omml2mathmlLib as any)?.default ?? (omml2mathmlLib as any);

    if (typeof fn === "function") {
      const result = fn(omml);
      
      if (typeof result === "string" && result.trim()) {
        return result;
      }
      if (result && typeof result === "object" && "toString" in result) {
        const str = String(result);
        if (str.trim()) {
          return str;
        }
      }
    }
  } catch (err) {
    console.warn("Error converting OMML to MathML:", err);
  }

  // Fallback: extract plain text from OMML
  const plainText = extractTextFromOmml(omml);
  if (plainText) {
    // Return as plain MathML with <mi> tags for identifiers
    return `<math xmlns="http://www.w3.org/1998/Math/MathML"><mi>${plainText}</mi></math>`;
  }

  return "";
}
