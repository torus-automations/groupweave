'use client';

import { useState, useEffect } from 'react';
import styles from '../styles/Game.module.css';

const rounds = [
  {
    images: [
      { id: 1, src: '/cactus_1.jpeg', name: 'Cactus' },
      { id: 2, src: '/cactus_2.jpeg', name: 'Cactus' },
    ],
    criteria: 'Which cactus is more aesthetic?',
  },
  {
    images: [
      { id: 3, src: '/dress_1.jpeg', name: 'Dress' },
      { id: 4, src: '/dress_2.jpeg', name: 'Dress' },
    ],
    criteria: 'Which dress would you wear?',
  },
];

export default function Home() {
  const [currentRound, setCurrentRound] = useState(0);
  const [selectedImage, setSelectedImage] = useState<number | null>(null);
  const [winner, setWinner] = useState<number | null>(null);
  const [countdown, setCountdown] = useState(10);
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  useEffect(() => {
    if (winner !== null) return;

    const timer = setInterval(() => {
      setCountdown((prevCountdown) => {
        if (prevCountdown === 1) {
          clearInterval(timer);
          // Auto-select a winner if none chosen
          if (selectedImage === null) {
            const randomWinner =
              rounds[currentRound].images[
                Math.floor(Math.random() * rounds[currentRound].images.length)
              ].id;
            setWinner(randomWinner);
          }
          return 0;
        }
        return prevCountdown - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [currentRound, winner, selectedImage]);

  const handleImageSelection = (id: number) => {
    if (winner === null) {
      setSelectedImage(id);
      setWinner(id);
    }
  };

  const handleNextRound = () => {
    if (currentRound < rounds.length - 1) {
      setCurrentRound(currentRound + 1);
      setSelectedImage(null);
      setWinner(null);
      setCountdown(10);
    } else {
      // Game over
      alert('Game over!');
    }
  };

  if (!isClient) {
    return null;
  }

  const { images, criteria } = rounds[currentRound];

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>Round {currentRound + 1}</h1>
      <p className={styles.subtitle}>
        {criteria}
      </p>
      {winner === null && <div className={styles.timer}>{countdown}</div>}
      <div className={styles.images}>
        {images.map((image) => (
          <div
            key={image.id}
            className={`${styles.imageWrapper} ${
              winner !== null && winner !== image.id ? styles.loser : ''
            } ${winner !== null && winner === image.id ? styles.winner : ''}`}>
            <img src={image.src} alt={image.name} className={styles.image} />
            <button
              onClick={() => handleImageSelection(image.id)}
              disabled={winner !== null}
              className={`${styles.btn} ${
                selectedImage === image.id ? styles.selectedBtn : ''
              }`}>
              Choose
            </button>
          </div>
        ))}
      </div>
      {winner !== null && (
        <>
          <p className={styles.result}>
            You chose the winner!
          </p>
          {currentRound < rounds.length - 1 && (
            <button onClick={handleNextRound} className={styles.nextBtn}>
              Next Round
            </button>
          )}
        </>
      )}
      <p className={styles.tagline}>
        This is a game of subjective choices. There are no right or wrong answers.
      </p>
    </div>
  );
}
