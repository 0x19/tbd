"use client";

import type {
  OryCardAuthMethodListItemProps,
  OryFlowComponentOverrides,
  OryMessageContentProps,
  OryNodeAnchorProps,
  OryNodeButtonProps,
  OryNodeCheckboxProps,
  OryNodeConsentScopeCheckboxProps,
  OryNodeInputProps,
  OryNodeLabelProps,
  OryNodeSsoButtonProps,
  OryNodeTextProps,
} from "@ory/elements-react";
import { useOryConfiguration, useOryFlow } from "@ory/elements-react";
import type { PropsWithChildren } from "react";

import { Logo } from "@/components/logo";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

// Ory Elements owns the flows (which nodes to show, submission, redirects); these
// overrides render them with the kit's primitives so the pages look like the rest
// of the product. Names and values on inputs and buttons are Kratos' own, which
// is what the browser checks (e2e/auth.mjs) and any script rely on.

const copy: Record<string, { title: string; description: string }> = {
  login: { title: "Sign in", description: "Use your email and password, a passkey, or Google." },
  registration: { title: "Create your account", description: "A password, a passkey, or Google." },
  recovery: { title: "Recover your account", description: "We send a one-time code to your email." },
  verification: { title: "Verify your email", description: "Enter the code we sent you." },
  settings: { title: "Account settings", description: "Change how you sign in." },
  oauth2_consent: { title: "Allow access?", description: "An application wants to use your account." },
};

const methodLabels: Record<string, string> = {
  password: "Password",
  passkey: "Passkey",
  webauthn: "Security key",
  code: "Email code",
  totp: "Authenticator app",
  lookup_secret: "Recovery code",
  oidc: "Social sign-in",
  saml: "Single sign-on",
};

function CardRoot({ children }: PropsWithChildren) {
  return <Card className="w-full max-w-sm">{children}</Card>;
}

function CardHead() {
  const { flowType } = useOryFlow();
  const text = copy[flowType] ?? { title: "tbd", description: "" };
  return (
    <CardHeader className="gap-2">
      <Logo width={28} height={28} className="mb-2" />
      <CardTitle className="text-xl">{text.title}</CardTitle>
      {text.description ? <CardDescription>{text.description}</CardDescription> : null}
    </CardHeader>
  );
}

function CardBody({ children }: PropsWithChildren) {
  return <CardContent className="grid gap-4">{children}</CardContent>;
}

function CardFoot() {
  const { flowType } = useOryFlow();
  const { project } = useOryConfiguration();
  const links: Record<string, { text: string; label: string; href: string }[]> = {
    login: [
      { text: "No account yet?", label: "Create one", href: project.registration_ui_url },
      ...(project.recovery_enabled
        ? [{ text: "Forgot your password?", label: "Recover it", href: project.recovery_ui_url }]
        : []),
    ],
    registration: [{ text: "Already have an account?", label: "Sign in", href: project.login_ui_url }],
    recovery: [{ text: "Remembered it?", label: "Sign in", href: project.login_ui_url }],
    verification: [{ text: "Done here?", label: "Sign in", href: project.login_ui_url }],
    settings: [{ text: "", label: "Back", href: project.default_redirect_url }],
  };
  const items = links[flowType] ?? [];
  if (!items.length) return null;
  return (
    <CardFooter className="text-muted-foreground flex flex-col items-start gap-1 text-sm">
      {items.map((l) => (
        <p key={l.href}>
          {l.text ? `${l.text} ` : ""}
          <a className="text-foreground font-medium underline underline-offset-4" href={l.href}>
            {l.label}
          </a>
        </p>
      ))}
    </CardFooter>
  );
}

function CardDivider() {
  return (
    <div className="relative my-1">
      <Separator />
      <span className="bg-card text-muted-foreground absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 px-2 text-xs uppercase">
        or
      </span>
    </div>
  );
}

function AuthMethodList({ children }: PropsWithChildren) {
  return <div className="grid gap-2">{children}</div>;
}

function AuthMethodListItem({ onClick, group, disabled }: OryCardAuthMethodListItemProps) {
  return (
    <Button
      type="button"
      variant="outline"
      className="w-full justify-start"
      onClick={onClick}
      disabled={disabled}
    >
      {methodLabels[group] ?? group}
    </Button>
  );
}

function FormRoot({ className, children, ...props }: React.ComponentPropsWithoutRef<"form">) {
  return (
    <form {...props} className={cn("grid gap-4", className)}>
      {children}
    </form>
  );
}

function FormGroup({ children }: PropsWithChildren) {
  return <div className="grid gap-3">{children}</div>;
}

function SsoRoot({ children }: PropsWithChildren) {
  return <div className="grid gap-2">{children}</div>;
}

function labelOf(node: { meta?: { label?: { text?: string } } }, fallback = "") {
  return node.meta?.label?.text ?? fallback;
}

function NodeButton({ node, attributes, buttonProps, isSubmitting }: OryNodeButtonProps) {
  const secondary =
    attributes.name === "screen" || attributes.value === "previous" || attributes.type === "button";
  return (
    <Button
      {...buttonProps}
      variant={secondary ? "ghost" : "default"}
      className={cn("w-full", secondary && "text-muted-foreground")}
      disabled={buttonProps.disabled || isSubmitting}
    >
      {labelOf(node, String(attributes.value ?? "Continue"))}
    </Button>
  );
}

function SsoButton({ node, buttonProps, isSubmitting, provider }: OryNodeSsoButtonProps) {
  return (
    <Button
      {...buttonProps}
      variant="outline"
      className="w-full"
      disabled={buttonProps.disabled || isSubmitting}
    >
      {labelOf(node, `Continue with ${provider}`)}
    </Button>
  );
}

function NodeAnchor({ attributes }: OryNodeAnchorProps) {
  return (
    <a className="text-sm underline underline-offset-4" href={attributes.href}>
      {attributes.title.text}
    </a>
  );
}

function NodeInput({ attributes, inputProps }: OryNodeInputProps) {
  const { type: _type, ...rest } = inputProps as typeof inputProps & { type?: string };
  if (attributes.type === "hidden") {
    return <input {...rest} type="hidden" />;
  }
  return (
    <Input
      {...rest}
      type={attributes.type}
      autoComplete={attributes.autocomplete}
      required={attributes.required}
      disabled={attributes.disabled}
      placeholder={inputPlaceholder(attributes.name)}
    />
  );
}

function inputPlaceholder(name: string) {
  if (name === "identifier" || name === "traits.email") return "you@example.com";
  if (name === "traits.name") return "Your name";
  if (name === "code") return "123456";
  return undefined;
}

function NodeLabel({ attributes, node, fieldError, children }: OryNodeLabelProps) {
  const text = labelOf(node, attributes.name);
  const error = fieldError as { message?: string } | undefined;
  return (
    <div className="grid gap-2">
      <Label htmlFor={attributes.name}>{text}</Label>
      {children}
      {error?.message ? <p className="text-destructive text-sm">{error.message}</p> : null}
    </div>
  );
}

function NodeCheckbox({ attributes, node, inputProps }: OryNodeCheckboxProps) {
  return (
    <label className="flex items-center gap-2 text-sm">
      <input
        {...inputProps}
        className="border-input accent-primary size-4 rounded"
        disabled={attributes.disabled}
      />
      {labelOf(node, attributes.name)}
    </label>
  );
}

function ConsentScopeCheckbox({
  attributes,
  node,
  inputProps,
  onCheckedChange,
}: OryNodeConsentScopeCheckboxProps) {
  return (
    <label className="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        name={inputProps.name}
        value={inputProps.value}
        checked={inputProps.checked}
        disabled={inputProps.disabled}
        onChange={(e) => onCheckedChange(e.target.checked)}
        className="border-input accent-primary size-4 rounded"
      />
      {labelOf(node as { meta?: { label?: { text?: string } } }, String(attributes.value))}
    </label>
  );
}

function NodeText({ node, attributes }: OryNodeTextProps) {
  return (
    <p className="text-muted-foreground text-sm">
      {labelOf(node, "")}
      {attributes.text?.text ? (
        <span className="text-foreground block font-mono">{attributes.text.text}</span>
      ) : null}
    </p>
  );
}

function MessageRoot({ children }: PropsWithChildren) {
  return <div className="grid gap-1">{children}</div>;
}

function MessageContent({ message }: OryMessageContentProps) {
  return (
    <p
      className={cn("text-sm", message.type === "error" ? "text-destructive" : "text-muted-foreground")}
      data-testid={`ory/message/${message.id}`}
    >
      {message.text}
    </p>
  );
}

function PageHeader() {
  return null;
}

export const oryComponents: OryFlowComponentOverrides = {
  Card: {
    Root: CardRoot,
    Header: CardHead,
    Content: CardBody,
    Footer: CardFoot,
    Logo: () => <Logo width={28} height={28} />,
    Divider: CardDivider,
    AuthMethodListContainer: AuthMethodList,
    AuthMethodListItem,
  },
  Form: { Root: FormRoot, Group: FormGroup, SsoRoot },
  Node: {
    Button: NodeButton,
    SsoButton,
    Anchor: NodeAnchor,
    Input: NodeInput,
    Label: NodeLabel,
    Checkbox: NodeCheckbox,
    ConsentScopeCheckbox,
    Text: NodeText,
  },
  Message: { Root: MessageRoot, Content: MessageContent },
  Page: { Header: PageHeader },
};
