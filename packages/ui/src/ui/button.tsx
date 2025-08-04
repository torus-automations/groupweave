'use client';

import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "../lib/utils.js"

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium ring-offset-background transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary-dark shadow-md hover:shadow-lg",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-md",
        outline:
          "border border-input bg-background hover:bg-surface-secondary hover:text-accent-foreground transition-colors",
        secondary:
          "bg-surface-secondary text-secondary-foreground hover:bg-surface-tertiary shadow-sm",
        ghost: "hover:bg-surface-secondary hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
        
        // Premium wallet variants
        primary: "bg-gradient-primary text-primary-foreground hover:shadow-xl transform hover:-translate-y-0.5 font-semibold",
        accent: "bg-gradient-accent text-accent-foreground hover:shadow-xl transform hover:-translate-y-0.5 font-semibold",
        minimal: "bg-transparent text-text-secondary hover:text-text-primary hover:bg-surface-secondary border-0 shadow-none font-medium",
        
        // Wallet-specific variants
        wallet: "bg-gradient-primary text-primary-foreground hover:shadow-2xl transform hover:-translate-y-1 font-semibold px-6 py-3 animate-fade-in",
        "wallet-outline": "border border-primary text-primary bg-background hover:bg-primary hover:text-primary-foreground transition-all duration-300 shadow-sm hover:shadow-lg",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        xl: "h-12 rounded-lg px-10 text-base",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }