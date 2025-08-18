'use client';

import { useState, useEffect } from 'react';
import styles from '../styles/Game.module.css';

interface Image {
  id: number;
  data: string; // Base64 encoded image
  votes: number;
}

interface Round {
  id: number;
  criteria: string;
  images: Image[];
}

export default function Home() {
  const [rounds, setRounds] = useState<Round[]>([]);
  const [currentRoundIndex, setCurrentRoundIndex] = useState(0);
  const [selectedImageId, setSelectedImageId] = useState<number | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isVoting, setIsVoting] = useState(false);

  useEffect(() => {
    const fetchRounds = async () => {
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
            setError("An unknown error occurred");
        }
      } finally {
        setIsLoading(false);
      }
    };

    fetchRounds();
  }, []);

  const handleImageSelection = async (imageId: number) => {
    if (selectedImageId !== null || isVoting) return;

    setIsVoting(true);
    const roundId = rounds[currentRoundIndex].id;

    try {
      const response = await fetch(`http://localhost:8000/rounds/${roundId}/vote`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ image_id: imageId }),
      });

      if (!response.ok) {
        throw new Error('Failed to submit vote');
      }

      setSelectedImageId(imageId);

      // Optimistically update the vote count locally
      const updatedRounds = [...rounds];
      const votedImage = updatedRounds[currentRoundIndex].images.find(img => img.id === imageId);
      if(votedImage) {
        votedImage.votes += 1;
      }
      setRounds(updatedRounds);


    } catch (err) {
        if (err instanceof Error) {
            alert(`Error: ${err.message}`);
        } else {
            alert("An unknown error occurred while voting.");
        }
    } finally {
        setIsVoting(false);
    }
  };

  const handleNextRound = () => {
    if (currentRoundIndex < rounds.length - 1) {
      setCurrentRoundIndex(currentRoundIndex + 1);
      setSelectedImageId(null);
    } else {
      alert('You have completed all rounds!');
    }
  };

  if (isLoading) {
    return <div className={styles.container}><p>Loading rounds...</p></div>;
  }

  if (error) {
    return <div className={styles.container}><p>Error: {error}</p></div>;
  }

  if (rounds.length === 0) {
    return <div className={styles.container}><p>No rounds available. Please create some rounds first.</p></div>;
  }

  const currentRound = rounds[currentRoundIndex];

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>Round {currentRoundIndex + 1} / {rounds.length}</h1>
      <p className={styles.subtitle}>{currentRound.criteria}</p>

      <div className={styles.images}>
        {currentRound.images.map((image) => (
          <div
            key={image.id}
            className={`${styles.imageWrapper} ${selectedImageId === image.id ? styles.selected : ''}`}>
            <img
              src={`data:image/jpeg;base64,${image.data}`}
              alt={`Image ${image.id}`}
              className={styles.image}
            />
            <button
              onClick={() => handleImageSelection(image.id)}
              disabled={selectedImageId !== null || isVoting}
              className={`${styles.btn} ${selectedImageId === image.id ? styles.selectedBtn : ''}`}>
              {isVoting && selectedImageId === image.id ? "Voting..." : "Choose"}
            </button>
            <p className={styles.voteCount}>{image.votes} votes</p>
          </div>
        ))}
      </div>

      {selectedImageId !== null && (
        <>
          <p className={styles.result}>Thank you for your vote!</p>
          {currentRoundIndex < rounds.length - 1 && (
            <button onClick={handleNextRound} className={styles.nextBtn}>
              Next Round
            </button>
          )}
        </>
      )}

      <p className={styles.tagline}>
        Pick the choice that you think satisfies the criteria.
      </p>
    </div>
  );
}
