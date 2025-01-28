import { Dropdown } from 'react-native-element-dropdown';
import ProtectedRoute from './ProtectedRoute';
import { AntDesign } from '@expo/vector-icons';
import { useState } from 'react';
import { StyleSheet, Text } from 'react-native';

export interface ProgramSelectorProps {
  className?: string;
  allowNone?: boolean;
  onChange?: (value: string | null) => void;
}
interface ProgramOption {
  label: string;
  value: string | null;
}
export const ProgramSelector = ({ allowNone, onChange }: ProgramSelectorProps) => {
  const programs: ProgramOption[] = [
    { label: 'Richmond Health And Wellness Program', value: 'RHWP' },
    { label: 'Mobile Health And Wellness Program', value: 'MHWP' },
  ];
  if (allowNone) {
    programs.push({ label: 'None', value: null });
  }
  const [value, setValue] = useState(null);
  const [isFocus, setIsFocus] = useState(false);

  const renderLabel = () => {
    if (value || isFocus) {
      return (
        <Text style={[styles.label, isFocus && { color: 'blue' }]}>Search Within Program</Text>
      );
    }
    return null;
  };
  return (
    <>
      {renderLabel()}
      <Dropdown
        style={[styles.dropdown, isFocus && { borderColor: 'blue' }]}
        placeholderStyle={styles.placeholderStyle}
        selectedTextStyle={styles.selectedTextStyle}
        inputSearchStyle={styles.inputSearchStyle}
        iconStyle={styles.iconStyle}
        data={programs}
        maxHeight={300}
        labelField="label"
        valueField="value"
        placeholder={!isFocus ? 'Select item' : '...'}
        value={value}
        onFocus={() => setIsFocus(true)}
        onBlur={() => setIsFocus(false)}
        onChange={(item) => {
          if (onChange) {
            onChange(item.value);
          }
          setValue(item.value);
          setIsFocus(false);
        }}
        renderLeftIcon={() => (
          <AntDesign
            style={styles.icon}
            color={isFocus ? 'blue' : 'black'}
            name="Safety"
            size={20}
          />
        )}
      />
    </>
  );
};
const styles = StyleSheet.create({
  container: {
    backgroundColor: 'white',
    padding: 16,
  },
  dropdown: {
    height: 50,
    borderColor: 'gray',
    borderWidth: 0.5,
    borderRadius: 8,
    paddingHorizontal: 8,
  },
  icon: {
    marginRight: 5,
  },
  label: {
    position: 'absolute',
    backgroundColor: 'white',
    left: 22,
    top: 8,
    zIndex: 999,
    paddingHorizontal: 8,
    fontSize: 14,
  },
  placeholderStyle: {
    fontSize: 16,
  },
  selectedTextStyle: {
    fontSize: 16,
  },
  iconStyle: {
    width: 20,
    height: 20,
  },
  inputSearchStyle: {
    height: 40,
    fontSize: 16,
  },
});
