// (tabs)/sessions.tsx
import { View, Text, FlatList, StyleSheet } from 'react-native';
import { useEffect, useState } from 'react';
import { openDb } from '@/db'; // your Drizzle expo-sqlite setup
import { sessions } from '@/db/drizzle/schema';
import { sql } from 'drizzle-orm';

type Session = typeof sessions.$inferSelect;

export default function SessionsScreen() {
  const [allSessions, setAllSessions] = useState<Session[]>([]);

  useEffect(() => {
    const db = openDb();

    try {
      const data: Session[] = db.select().from(sessions).all();
      // console.log(data);
      setAllSessions(data);
    } catch (err) {
      console.error('Failed to fetch sessions:', err);
    }
  }, []);

  const renderItem = ({ item }: { item: Session }) => {
    // console.log(item.endTime);
    // console.log(new Date(Number(item.endTime)));

    return (
      <View className="mb-5 rounded-xl bg-gray-200 p-6">
        <Text>ID: {item.id}</Text>
        <Text style={styles.text}>User: {item.userId}</Text>
        <Text style={styles.text}>Category: {item.categoryId ?? '-'}</Text>
        <Text style={styles.text}>
          Start: {new Date(Number(item.startTime)).toLocaleTimeString()}
        </Text>
        <Text style={styles.text}>End:{new Date(Number(item.endTime)).toLocaleTimeString()}</Text>
        <Text style={styles.text}>Kind: {item.kind}</Text>
        <Text style={styles.text}>Completed: {item.completed ? 'Yes' : 'No'}</Text>
      </View>
    );
  };

  return (
    <View className="bg-background">
      <FlatList
        className="p-8"
        data={allSessions}
        keyExtractor={(item) => item.id.toString()}
        renderItem={renderItem}
        ListEmptyComponent={<Text>No sessions yet</Text>}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  sessionCard: {
    padding: 12,
    marginBottom: 12,
    borderRadius: 8,
    backgroundColor: '#f1f1f1',
  },
  text: { fontSize: 14 },
});
