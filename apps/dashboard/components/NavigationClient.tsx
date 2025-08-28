'use client';

import { Navigation, NavigationItem } from '@repo/ui/navigation';

export default function NavigationClient() {
  const dashboardNavItems: NavigationItem[] = [
    { name: "Overview", href: "/" },
    { name: "Analytics", href: "/analytics" },
    { name: "Projects", href: "/projects" },
    { name: "Settings", href: "/settings" },
  ];

  return <Navigation navItems={dashboardNavItems} />;
}