'use client';

import { Navigation, NavigationItem } from '@repo/ui/navigation';

export default function NavigationClient() {
  const creationNavItems: NavigationItem[] = [
    { name: "Create", href: "/" },
    { name: "Templates", href: "/templates" },
    { name: "My Projects", href: "/projects" },
    { name: "Collaborate", href: "/collaborate" },
  ];

  return <Navigation navItems={creationNavItems} />;
}