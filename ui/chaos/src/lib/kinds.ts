// Service kinds as the API describes them (`GET /overview` → `kinds`,
// docs/chaos/kinds.md). Pages derive their forms, filters and copy from these
// descriptors instead of naming engine, protocol or ledger; a kind scaffolded by
// `tbd new service` shows up here without a UI change.
import type { KindDescriptor, KindField } from "@/lib/api/schema";

/** A descriptor for a kind the API did not list (an old server, a stale overview). */
export function fallbackKind(name: string): KindDescriptor {
  return {
    name,
    label: name.charAt(0).toUpperCase() + name.slice(1),
    plural: `${name}s`,
    surface: "",
    fault: false,
    store_fault: false,
    counters: false,
    load_target: false,
    addable: false,
    target: false,
    dependency_kind: null,
    fields: [],
  };
}

/** The descriptor named `name`, or a synthesised one. */
export function kindOf(kinds: KindDescriptor[], name: string): KindDescriptor {
  return kinds.find((k) => k.name === name) ?? fallbackKind(name);
}

/** The kind whose topology table is `plural`, or a synthesised one. */
export function kindOfPlural(kinds: KindDescriptor[], plural: string): KindDescriptor {
  return kinds.find((k) => k.plural === plural) ?? fallbackKind(plural.replace(/s$/, ""));
}

/** "Protocol base URL", "Engine gRPC URL": the label of a validate target input. */
export function targetLabel(kind: KindDescriptor): string {
  return kind.surface === "grpc" ? `${kind.label} gRPC URL` : `${kind.label} base URL`;
}

/** The kinds that have a validate target, in registry order. */
export function targetKinds(kinds: KindDescriptor[]): KindDescriptor[] {
  return kinds.filter((k) => k.target);
}

/** Kinds that take faults, counters, load or runtime adds, by capability. */
export function withCapability(
  kinds: KindDescriptor[],
  capability: "fault" | "counters" | "load_target" | "addable",
): KindDescriptor[] {
  return kinds.filter((k) => k[capability]);
}

/** "engines and ledgers": the kinds with a capability, as prose. */
export function listKinds(kinds: KindDescriptor[]): string {
  const names = kinds.map((k) => k.plural);
  if (names.length === 0) return "none";
  if (names.length === 1) return names[0];
  return `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

/** The value a field starts with in a form: its default, or the first instance of its kind. */
export function fieldInitial(field: KindField, instancesOfKind: string[]): string {
  if (field.kind === "instance_of") return instancesOfKind[0] ?? "";
  return field.default ?? "";
}

/** A `[stack.<plural>.<name>-2]` block per kind, for the scenario editor's snippets. */
export function stackSnippets(kinds: KindDescriptor[]): { title: string; toml: string }[] {
  const out: { title: string; toml: string }[] = [];
  for (const k of kinds) {
    const lines = [`[stack.${k.plural}.${k.name}-2]`];
    for (const f of k.fields) {
      const value =
        f.kind === "instance_of" ? `${f.of_kind ?? "engine"}-1` : f.kind === "duration" ? "100ms" : "";
      if (f.required || f.default !== null) lines.push(`${f.name} = "${f.default ?? value}"`);
    }
    out.push({ title: k.label, toml: `${lines.join("\n")}\n` });
    if (k.fault) {
      out.push({
        title: `${k.label} with an initial behaviour`,
        toml: `${lines.join("\n")}\n[stack.${k.plural}.${k.name}-2.behavior]\ntype = "slow"\nlatency = "20ms"\n`,
      });
    }
  }
  return out;
}
