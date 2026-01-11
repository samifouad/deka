import { nLayout } from "@/types";

export default async function SettingsLayout({children}: nLayout) {
    return (
      <section>
        <h2>Settings</h2>
        {children}
      </section>
    )
  }