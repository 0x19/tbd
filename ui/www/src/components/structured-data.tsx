import { company, url } from "@/data/site";

/**
 * JSON-LD for search engines: the organisation the site belongs to, the person
 * behind it, and the site itself. Every value comes from `data/site.ts`, so it
 * says exactly what the pages say, and nothing the pages do not (no employer,
 * no figures). A TODO field is simply absent rather than rendered empty.
 */
export function StructuredData() {
  const organization = {
    "@type": "Organization",
    "@id": `${url}/#organization`,
    name: company.name,
    legalName: company.legalName,
    url,
    email: company.email,
    foundingDate: company.founded === null ? undefined : String(company.founded),
    founder: { "@id": `${url}/#person` },
    logo: `${url}/brand/mark.svg`,
  };
  const person = {
    "@type": "Person",
    "@id": `${url}/#person`,
    name: company.person,
    url: `${url}/about/`,
    email: company.email,
    jobTitle: company.title,
    affiliation: { "@id": `${url}/#organization` },
    sameAs: [company.github, company.x, company.linkedin],
  };
  const website = {
    "@type": "WebSite",
    "@id": `${url}/#website`,
    name: company.name,
    url,
    publisher: { "@id": `${url}/#organization` },
  };
  const graph = { "@context": "https://schema.org", "@graph": [organization, person, website] };
  // `</script>` inside a string would end the element early; the escape keeps the JSON valid.
  const json = JSON.stringify(graph).replace(/</g, "\\u003c");
  return <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: json }} />;
}
