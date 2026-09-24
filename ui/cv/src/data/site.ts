/**
 * The facts the chrome needs, mirrored from the public site's data file
 * (`ui/www/src/data/site.ts`): this app is the public site's gated room, so
 * its header and footer are the public site's, with the pages linking back.
 * A copy change there is a copy change here.
 */
export const publicUrl = "https://inorbit.hr";

export const company = {
  name: "InOrbit",
  legalName: "InOrbit d.o.o.",
  person: "Nevio Vesic",
  tagline: "Backend and blockchain systems.",
  email: "nevio@inorbit.hr",
  city: "Rijeka and Zagreb, Croatia",
  github: "https://github.com/0x19",
  x: "https://x.com/vesicnevio",
  linkedin: "https://www.linkedin.com/in/neviovesic/",
  founded: 2018,
} as const;

/** The public site's pages, as the header and footer list them here; `key` names the label in `nav.<key>`. */
export const publicNav = [
  // The lab (`/lab/`) is admins-only on the public site until it is published; it
  // joins this list with that flip.
  { href: `${publicUrl}/work/`, label: "Work", key: "work" },
  { href: `${publicUrl}/playgrounds/`, label: "Play", key: "play" },
  { href: `${publicUrl}/about/`, label: "About", key: "about" },
  { href: `${publicUrl}/contact/`, label: "Contact", key: "contact" },
] as const;
