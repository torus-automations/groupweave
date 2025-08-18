import { render, screen } from '@testing-library/react';
import { Navigation } from './Navigation';
import React from 'react';

describe('Navigation', () => {
  it('renders the connect button when not signed in', () => {
    render(<Navigation accountId={null} onConnect={() => {}} onDisconnect={() => {}} />);
    expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
  });

  it('renders the formatted account id when signed in with a long name', () => {
    render(<Navigation accountId="very-long-account-id-that-should-be-truncated.near" onConnect={() => {}} onDisconnect={() => {}} />);
    expect(screen.getByText('very-long-...near')).toBeInTheDocument();
  });

  it('renders the full account id when signed in with a short name', () => {
    render(<Navigation accountId="short.near" onConnect={() => {}} onDisconnect={() => {}} />);
    expect(screen.getByText('short.near')).toBeInTheDocument();
  });
});
