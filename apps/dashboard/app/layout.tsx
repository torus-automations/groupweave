import type { Metadata } from "next";
import localFont from "next/font/local";
import { ExpandableButton } from "@repo/ui/ui/expandable-button";
import { Footer } from "@repo/ui/footer";
import NavigationClient from "../components/NavigationClient";
import "./globals.css";

const geistSans = localFont({
  src: "./fonts/GeistVF.woff",
  variable: "--font-geist-sans",
});
const geistMono = localFont({
  src: "./fonts/GeistMonoVF.woff",
  variable: "--font-geist-mono",
});

export const metadata: Metadata = {
  title: "GroupWeave Dashboard",
  description: "Manage and monitor your collaborative projects",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        <div className="min-h-screen flex flex-col" style={{ backgroundColor: 'hsl(var(--background))' }}>
          <NavigationClient />
          <main className="flex-1 flex flex-col" role="main">
            {children}
          </main>
          <Footer />
          <ExpandableButton />
        </div>
      </body>
    </html>
  );
}
