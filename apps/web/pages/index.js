import Head from 'next/head';
import { useState, useEffect } from 'react';
import styles from '../styles/Game.module.css';

const rounds = [
  ['/cactus_1.jpeg', '/cactus_2.jpeg'],
  ['/dress_1.jpeg', '/dress_2.jpeg']
];

export default function Home() {
  // duration of each round in seconds
  const ROUND_DURATION = 10;

  const [round, setRound] = useState(0);
  const [timeLeft, setTimeLeft] = useState(ROUND_DURATION);
  const [selected, setSelected] = useState(null); // 0 or 1
  const [choicesCount, setChoicesCount] = useState([0, 0]);
  const [winner, setWinner] = useState(null); // 0 or 1
  const [showResult, setShowResult] = useState(false);
  const CRITERIA = 'NOT AI generated';

  // countdown / end-of-round logic
  useEffect(() => {
    if (showResult) return; // stop when we are showing results

    if (timeLeft === 0) {
      // decide winner based on selections so far (placeholder for backend aggregation)
      const winIdx = choicesCount[0] >= choicesCount[1] ? 0 : 1;
      setWinner(winIdx);
      setShowResult(true);
      return;
    }

    const id = setInterval(() => {
      setTimeLeft((t) => t - 1);
    }, 1000);
    return () => clearInterval(id);
  }, [timeLeft, showResult, choicesCount]);

  const handleSelect = (idx) => {
    if (selected !== null || showResult) return;
    setSelected(idx);
    setChoicesCount((prev) => {
      const next = [...prev];
      next[idx] += 1;
      return next;
    });
  };

  const nextRound = () => {
    setRound((r) => r + 1);
    setTimeLeft(ROUND_DURATION);
    setSelected(null);
    setChoicesCount([0, 0]);
    setWinner(null);
    setShowResult(false);
  };

  return (
    <>
      <Head>
        <title>GroupWeave</title>
        <meta name="description" content="Pick the more realistic image!" />
      </Head>

      <div className={styles.container}>
        <div className={styles.tagline}>GroupWeave&nbsp;Co-Creation</div>
        <h2 className={styles.subtitle}>
          Pick the image(s) &nbsp;
          <span className={styles.criteria}>{CRITERIA}</span>
        </h2>
        {/* countdown */}
        <div className={styles.timer}>{timeLeft}</div>

        {/* image options */}
        <div className={styles.images}>
          {rounds[round].map((src, idx) => (
            <div
              key={src}
              className={`${styles.imageWrapper} ${
                showResult ? (idx === winner ? styles.winner : styles.loser) : ''
              }`}
            >
              <img src={src} alt={`Option ${idx === 0 ? 'A' : 'B'}`} className={styles.image} />
              <button
                className={`${styles.btn} ${selected === idx ? styles.selectedBtn : ''}` }
                onClick={() => handleSelect(idx)}
                disabled={selected !== null || showResult}
              >
                {`Select ${idx === 0 ? 'A' : 'B'}`}
              </button>
            </div>
          ))}
        </div>

        {showResult && (
          <div className={styles.result}>
            {selected === winner ? 'You won!' : 'Better luck next time'} – Majority chose{' '}
            {winner === 0 ? 'A' : 'B'}
          </div>
        )}

        {showResult && round < rounds.length - 1 && (
          <button className={styles.nextBtn} onClick={nextRound}>
            Next Round
          </button>
        )}

        {showResult && round === rounds.length - 1 && (
          <div className={styles.result}>Thanks for playing!</div>
        )}
      </div>
    </>
  );
}
