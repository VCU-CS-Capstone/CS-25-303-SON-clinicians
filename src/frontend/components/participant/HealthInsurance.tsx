import { AntDesign } from '@expo/vector-icons';
import { useState } from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { Dropdown, MultiSelect } from 'react-native-element-dropdown';
import { HealthInsurance } from '~/lib/types/participant';

export interface InsuranceSelectorOptions {
  className?: string;
  value: HealthInsurance[];
  onChange?: (value: HealthInsurance[] | null) => void;
}
interface InsuranceOptions {
  label: string;
  value: HealthInsurance;
}
export const ShowInsurances = ({ insurances }: { insurances: HealthInsurance[] }) => {
  return (
    <View>
      {insurances.map((insurance) => (
        <View key={insurance} style={styles.item}>
          <Text style={styles.selectedTextStyle}>{HealthInsurance.fullName(insurance)}</Text>
        </View>
      ))}
    </View>
  );
};

export const HealthInsuranceSelector = ({ value, onChange }: InsuranceSelectorOptions) => {
  const options: InsuranceOptions[] = [];

  for (const program in HealthInsurance) {
    options.push({
      label: HealthInsurance.fullName(program as HealthInsurance),
      value: program as HealthInsurance,
    });
  }

  const [isFocus, setIsFocus] = useState(false);

  const renderItem = (item: InsuranceOptions) => {
    return (
      <View style={styles.item}>
        <Text style={styles.selectedTextStyle}>{item.label}</Text>
        <AntDesign style={styles.icon} color="black" name="Safety" size={20} />
      </View>
    );
  };

  return (
    <>
      <MultiSelect
        style={styles.dropdown}
        placeholderStyle={styles.placeholderStyle}
        selectedTextStyle={styles.selectedTextStyle}
        inputSearchStyle={styles.inputSearchStyle}
        iconStyle={styles.iconStyle}
        data={options}
        labelField="label"
        valueField="value"
        placeholder="Select item"
        value={value}
        search
        searchPlaceholder="Search..."
        onChange={(item) => {
          if (onChange) {
            onChange(item as HealthInsurance[]);
          }
        }}
        renderLeftIcon={() => (
          <AntDesign style={styles.icon} color="black" name="Safety" size={20} />
        )}
        renderItem={renderItem}
        renderSelectedItem={(item, unSelect) => (
          <TouchableOpacity onPress={() => unSelect && unSelect(item)}>
            <View style={styles.selectedStyle}>
              <Text style={styles.textSelectedStyle}>{item.label}</Text>
              <AntDesign color="black" name="delete" size={17} />
            </View>
          </TouchableOpacity>
        )}
      />
    </>
  );
};
const styles = StyleSheet.create({
  container: { padding: 16 },
  dropdown: {
    height: 50,
    backgroundColor: 'white',
    borderRadius: 12,
    padding: 12,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.2,
    shadowRadius: 1.41,

    elevation: 2,
  },
  placeholderStyle: {
    fontSize: 16,
  },
  selectedTextStyle: {
    fontSize: 14,
  },
  iconStyle: {
    width: 20,
    height: 20,
  },
  inputSearchStyle: {
    height: 40,
    fontSize: 16,
  },
  icon: {
    marginRight: 5,
  },
  item: {
    padding: 17,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  selectedStyle: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    borderRadius: 14,
    backgroundColor: 'white',
    shadowColor: '#000',
    marginTop: 8,
    marginRight: 12,
    paddingHorizontal: 12,
    paddingVertical: 8,
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.2,
    shadowRadius: 1.41,

    elevation: 2,
  },
  textSelectedStyle: {
    marginRight: 5,
    fontSize: 16,
  },
});
