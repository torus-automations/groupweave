'use client';

import React, { useState, useRef } from 'react';
// import { Button } from '@repo/ui';
import type { CreateRoundRequest, Round } from '@groupweave/common-types';

export default function Home(): React.JSX.Element {
  const [criteria, setCriteria] = useState('');
  const [image1, setImage1] = useState<File | null>(null);
  const [image2, setImage2] = useState<File | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [message, setMessage] = useState('');

  const criteriaRef = useRef<HTMLInputElement>(null);
  const image1Ref = useRef<HTMLInputElement>(null);
  const image2Ref = useRef<HTMLInputElement>(null);

  const toBase64 = (file: File): Promise<string> =>
    new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.readAsDataURL(file);
      reader.onload = () => {
        const result = reader.result;
        if (typeof result === 'string') {
          const base64 = result.split(',')[1];
          if (base64) {
            resolve(base64);
          } else {
            reject(new Error('Invalid file format'));
          }
        } else {
          reject(new Error('Failed to read file'));
        }
      };
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

      const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000';
      const response = await fetch(`${apiUrl}/rounds`, {
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
      // Reset file inputs using refs
      if (criteriaRef.current) criteriaRef.current.value = '';
      if (image1Ref.current) image1Ref.current.value = '';
      if (image2Ref.current) image2Ref.current.value = '';

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
          <label htmlFor="criteria" style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>Criteria</label>
          <input
            ref={criteriaRef}
            id="criteria"
            type="text"
            placeholder="e.g. Which is more aesthetic?"
            value={criteria}
            onChange={(e) => setCriteria(e.target.value)}
            disabled={isSubmitting}
            style={{ width: '100%', padding: '8px', border: '1px solid #ccc', borderRadius: '4px' }}
          />
        </div>
        <div style={{ marginBottom: '15px' }}>
          <label htmlFor="image1" style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>Image 1</label>
          <input
            ref={image1Ref}
            id="image1"
            type="file"
            accept="image/*"
            onChange={(e) => setImage1(e.target.files?.[0] || null)}
            disabled={isSubmitting}
            style={{ width: '100%', padding: '8px', border: '1px solid #ccc', borderRadius: '4px' }}
          />
        </div>
        <div style={{ marginBottom: '15px' }}>
          <label htmlFor="image2" style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>Image 2</label>
          <input
            ref={image2Ref}
            id="image2"
            type="file"
            accept="image/*"
            onChange={(e) => setImage2(e.target.files?.[0] || null)}
            disabled={isSubmitting}
            style={{ width: '100%', padding: '8px', border: '1px solid #ccc', borderRadius: '4px' }}
          />
        </div>
        <button
          type="submit"
          disabled={isSubmitting}
          style={{
            padding: '12px 24px',
            backgroundColor: '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isSubmitting ? 'not-allowed' : 'pointer',
            opacity: isSubmitting ? 0.6 : 1
          }}
        >
          {isSubmitting ? 'Creating...' : 'Create Round'}
        </button>
      </form>
      {message && <p style={{ marginTop: '20px', textAlign: 'center' }}>{message}</p>}
    </div>
  );
}
