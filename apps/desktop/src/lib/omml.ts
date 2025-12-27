import omml2mathmlLib from "omml2mathml";

export function ommlToMathml(omml: string): string {
  try {
    const fn = (omml2mathmlLib as any)?.default ?? (omml2mathmlLib as any);

    if (typeof fn === "function") {
      const result = fn(omml);
      if (typeof result === "string") {
        return result;
      }
      if (result && typeof result === "object" && "toString" in result) {
        return String(result);
      }
    }
  } catch {
    // fall through
  }

  return "";
}
