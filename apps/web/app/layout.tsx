import type { Metadata } from "next";
import localFont from "next/font/local";
import { Menubar, MenubarMenu, MenubarTrigger, MenubarContent, MenubarItem } from "@repo/ui";
import "../styles/globals.css";

const geistSans = localFont({
  src: "./fonts/GeistVF.woff",
  variable: "--font-geist-sans",
});
const geistMono = localFont({
  src: "./fonts/GeistMonoVF.woff",
  variable: "--font-geist-mono",
});

export const metadata: Metadata = {
  title: "GroupWeave Co-Creation",
  description: "Interactive group decision making game",
  icons: {
    icon: '/favicon.ico',
  },
};

export const viewport = {
  width: 'device-width',
  initialScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        <div className="min-h-screen bg-background flex flex-col">
          <header className="w-full p-4 flex justify-center border-b border-border bg-card" role="banner">
            <Menubar>
              <MenubarMenu>
                <MenubarTrigger>File</MenubarTrigger>
                <MenubarContent>
                  <MenubarItem>New</MenubarItem>
                  <MenubarItem>Open</MenubarItem>
                  <MenubarItem>Save</MenubarItem>
                </MenubarContent>
              </MenubarMenu>
              <MenubarMenu>
                <MenubarTrigger>Edit</MenubarTrigger>
                <MenubarContent>
                  <MenubarItem>Undo</MenubarItem>
                  <MenubarItem>Redo</MenubarItem>
                  <MenubarItem>Cut</MenubarItem>
                </MenubarContent>
              </MenubarMenu>
              <MenubarMenu>
                <MenubarTrigger>View</MenubarTrigger>
                <MenubarContent>
                  <MenubarItem>Zoom In</MenubarItem>
                  <MenubarItem>Zoom Out</MenubarItem>
                  <MenubarItem>Full Screen</MenubarItem>
                </MenubarContent>
              </MenubarMenu>
            </Menubar>
          </header>
          <main className="flex-1 flex flex-col" role="main">
            {children}
          </main>
        </div>
      </body>
    </html>
  );
}
