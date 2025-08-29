import React from "react";
import { Globe, MessageSquare, Send, Linkedin } from "lucide-react";

const socialLinks = [
  {
    name: "Website",
    url: "https://www.torus-automations.xyz/",
    icon: <Globe className="h-6 w-6" />,
  },
  {
    name: "Discord",
    url: "https://discord.gg/wgN9HhUM",
    icon: <MessageSquare className="h-6 w-6" />,
  },
  {
    name: "Telegram",
    url: "https://t.me/torusautomations",
    icon: <Send className="h-6 w-6" />,
  },
  {
    name: "LinkedIn",
    url: "https://www.linkedin.com/company/torus-automations/",
    icon: <Linkedin className="h-6 w-6" />,
  },
];

export function Footer() {
  return (
    <footer className="bg-background border-t">
      <div className="container mx-auto py-6 px-4 md:px-6">
        <div className="flex flex-col md:flex-row items-center justify-between">
          <p className="text-sm text-muted-foreground mb-4 md:mb-0">
            © {new Date().getFullYear()} Torus Automations. All rights reserved.
          </p>
          <div className="flex items-center space-x-4">
            {socialLinks.map((link) => (
              <a
                key={link.name}
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-foreground transition-colors"
                aria-label={link.name}
              >
                {link.icon}
              </a>
            ))}
          </div>
        </div>
      </div>
    </footer>
  );
}
