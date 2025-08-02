import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Link } from 'expo-router';
import { SafeAreaView } from 'react-native-safe-area-context';

export default function HomeScreen() {
  return (
    <SafeAreaView style={styles.container}>
      <View style={styles.content}>
        <Text style={styles.title}>GroupWeave</Text>
        <Text style={styles.subtitle}>Decentralized Governance Platform</Text>
        
        <View style={styles.buttonContainer}>
          <Link href="/voting" asChild>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>🗳️ Voting</Text>
            </TouchableOpacity>
          </Link>
          
          <Link href="/staking" asChild>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>💰 Staking</Text>
            </TouchableOpacity>
          </Link>
          
          <Link href="/profile" asChild>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>👤 Profile</Text>
            </TouchableOpacity>
          </Link>
        </View>
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  content: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    marginBottom: 8,
    color: '#333',
  },
  subtitle: {
    fontSize: 16,
    color: '#666',
    marginBottom: 40,
    textAlign: 'center',
  },
  buttonContainer: {
    width: '100%',
    gap: 16,
  },
  button: {
    backgroundColor: '#007AFF',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonText: {
    color: 'white',
    fontSize: 18,
    fontWeight: '600',
  },
});