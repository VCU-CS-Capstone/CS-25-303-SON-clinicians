import { View, Text } from 'react-native';
import { StyleSheet } from 'react-native';

const LabelAndItem = ({ label, children }: { label: string; children: React.ReactNode }) => {
  return (
    <View style={styles.container}>
      <Text style={styles.label}>{label}</Text>
      {children}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
    borderWidth: 2,
    borderStyle: 'solid',
    borderColor: '#FFCDD2',
  },
  label: {
    fontSize: 24,
    color: 'black',
    fontWeight: 'bold',
  },
});
export default LabelAndItem;
