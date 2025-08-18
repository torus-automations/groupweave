'use client';

import { Button } from '@repo/ui/button';
import { Input } from '@repo/ui/ui/input';
import { Label } from '@repo/ui/ui/label';
import { useState } from 'react';

export default function Home(): JSX.Element {
  const [criteria, setCriteria] = useState('');
  const [image1, setImage1] = useState<File | null>(null);
  const [image2, setImage2] = useState<File | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [message, setMessage] = useState('');

  const toBase64 = (file: File): Promise<string> =>
    new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.readAsDataURL(file);
      reader.onload = () => resolve((reader.result as string).split(',')[1]);
      reader.onerror = (error) => reject(error);
    });

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!criteria || !image1 || !image2) {
      setMessage('Please fill out all fields.');
      return;
    }

    setIsSubmitting(true);
    setMessage('Creating round...');

    try {
      const image1_data = await toBase64(image1);
      const image2_data = await toBase64(image2);

      const response = await fetch('http://localhost:8000/rounds', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          criteria,
          image1_data,
          image2_data,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.detail || 'Failed to create round.');
      }

      const result = await response.json();
      setMessage(`Round created successfully! ID: ${result.id}`);
      setCriteria('');
      setImage1(null);
      setImage2(null);
      // Reset file inputs
      (document.getElementById('criteria') as HTMLInputElement).value = '';
      (document.getElementById('image1') as HTMLInputElement).value = '';
      (document.getElementById('image2') as HTMLInputElement).value = '';

    } catch (error) {
      console.error(error);
      if (error instanceof Error) {
        setMessage(`An error occurred: ${error.message}`);
      } else {
        setMessage('An unknown error occurred while creating the round.');
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div style={{ padding: '20px', maxWidth: '500px', margin: 'auto' }}>
      <h1 style={{ textAlign: 'center', marginBottom: '20px' }}>Create a new Round</h1>
      <form onSubmit={handleSubmit}>
        <div style={{ marginBottom: '15px' }}>
          <Label htmlFor="criteria">Criteria</Label>
          <Input
            id="criteria"
            type="text"
            placeholder="e.g. Which is more aesthetic?"
            value={criteria}
            onChange={(e) => setCriteria(e.target.value)}
            disabled={isSubmitting}
          />
        </div>
        <div style={{ marginBottom: '15px' }}>
          <Label htmlFor="image1">Image 1</Label>
          <Input
            id="image1"
            type="file"
            accept="image/*"
            onChange={(e) => setImage1(e.target.files ? e.target.files[0] : null)}
            disabled={isSubmitting}
          />
        </div>
        <div style={{ marginBottom: '15px' }}>
          <Label htmlFor="image2">Image 2</Label>
          <Input
            id="image2"
            type="file"
            accept="image/*"
            onChange={(e) => setImage2(e.target.files ? e.target.files[0] : null)}
            disabled={isSubmitting}
          />
        </div>
        <Button appName="creation" type="submit" disabled={isSubmitting}>
          {isSubmitting ? 'Creating...' : 'Create Round'}
        </Button>
      </form>
      {message && <p style={{ marginTop: '20px', textAlign: 'center' }}>{message}</p>}
    </div>
  );
}
