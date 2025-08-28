import type { Metadata } from "next";
import localFont from "next/font/local";
import NavigationClient from "../components/NavigationClient";
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
        <div className="min-h-screen flex flex-col" style={{ backgroundColor: 'hsl(var(--background))' }}>
          <NavigationClient />
          <main className="flex-1 flex flex-col" role="main">
            {children}
          </main>
        </div>
      </body>
    </html>
  );
}
