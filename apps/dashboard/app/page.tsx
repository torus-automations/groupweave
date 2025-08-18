'use client';

import { useState, useEffect } from 'react';
import { Card } from '@repo/ui/card';
import { Button } from '@repo/ui/button';

interface Image {
  id: number;
  data: string;
  votes: number;
}

interface Round {
  id: number;
  criteria: string;
  images: Image[];
}

export default function Home(): JSX.Element {
  const [rounds, setRounds] = useState<Round[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchRounds = async () => {
    setIsLoading(true);
    try {
      const response = await fetch('http://localhost:8000/rounds');
      if (!response.ok) {
        throw new Error('Failed to fetch rounds');
      }
      const data: Round[] = await response.json();
      setRounds(data);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('An unknown error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchRounds();
  }, []);

  return (
    <div style={{ padding: '20px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h1>Rounds Dashboard</h1>
        <Button appName="dashboard" onClick={fetchRounds} disabled={isLoading}>
          {isLoading ? 'Refreshing...' : 'Refresh'}
        </Button>
      </div>

      {isLoading && <p>Loading dashboard...</p>}
      {error && <p style={{ color: 'red' }}>Error: {error}</p>}

      {!isLoading && !error && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '20px' }}>
          {rounds.map((round) => (
            <Card key={round.id} title={`Round #${round.id}`} href="#">
              <div>
                <p style={{ fontWeight: 'bold', marginBottom: '10px' }}>{round.criteria}</p>
                <div style={{ display: 'flex', justifyContent: 'space-around' }}>
                  {round.images.map(image => (
                    <div key={image.id} style={{ textAlign: 'center' }}>
                      <img
                        src={`data:image/jpeg;base64,${image.data}`}
                        alt={`Image ${image.id}`}
                        style={{ width: '100px', height: '100px', objectFit: 'cover', borderRadius: '8px' }}
                      />
                      <p>{image.votes} votes</p>
                    </div>
                  ))}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
      {
        !isLoading && !error && rounds.length === 0 && <p>No rounds found.</p>
      }
    </div>
  );
}
