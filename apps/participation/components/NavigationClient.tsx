'use client';

import { Navigation, NavigationItem } from '@repo/ui/navigation';

export default function NavigationClient() {
  const participationNavItems: NavigationItem[] = [
    { name: "Co-Create", href: "/" },
    { name: "My Decisions", href: "/decisions" },
    { name: "Group Progress", href: "/progress" },
    { name: "How It Works", href: "/guide" },
  ];

  return <Navigation navItems={participationNavItems} />;
}