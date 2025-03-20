import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import LabelAndItem from '~/components/LabelAndItem';

import ProtectedRoute from '~/components/ProtectedRoute';

import api from '~/lib/api';

import { Participant, ParticipantStatus } from '~/lib/types/participant';
import { BoxHeader, participantOverViewStyles as styles } from '~/components/participant/overview';
import { HealthOverviewBox } from '~/components/participant/overview/health';
import { ParticipantDemographicsBox } from '~/components/participant/overview/demographics';
import { NoDataScreen } from '~/components/NoDataScreen';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const router = useRouter();
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(participantNumberId);
      setParticipant(patient);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchPatient();
  }, []);

  return (
    <ProtectedRoute>
      <View>
        {error ? <Text>{error}</Text> : null}
        <ParticipantView participant={participant} />
      </View>
    </ProtectedRoute>
  );
}
function ParticipantView({ participant }: { participant: Participant | undefined }) {
  if (!participant) {
    return (
      <NoDataScreen title="No Participant Found" subtitle="No participant found with that ID" />
    );
  }
  return (
    <ScrollView>
      <View style={styles.fullWidth}>
        <Text style={styles.participantName}>
          {participant.first_name} {participant.last_name} ({participant.id})
        </Text>
      </View>
      <View style={styles.flexRowWrap}>
        <ParticipantBox {...participant} />
        <HealthOverviewBox particpantId={participant.id} />
        <ParticipantDemographicsBox particpantId={participant.id} />
      </View>
    </ScrollView>
  );
}

function ParticipantBox(participant: Participant) {
  return (
    <View style={styles.box}>
      <BoxHeader title="Participant" />
      <View style={styles.marginBottom}>
        <LabelAndItem label="Contact Info">
          <InlineSubLabelItem title="Phone Number 1" value={participant.phone_number_one} />
          <InlineSubLabelItem title="Phone Number 2" value={participant.phone_number_two} />
          <SubLabelItem title="Other" value={participant.other_contact} />
        </LabelAndItem>
        <LabelAndItem label="Status">
          <InlineSubLabelItem title="Status" value={ParticipantStatus.title(participant.status)} />
          <InlineSubLabelItem
            title="VCU Health Services Patient Status"
            value={participant.vcuhs_patient_status}
          />
        </LabelAndItem>
        <LabelAndItem label="Other Info">
          <InlineSubLabelItem
            title="Patient Since"
            value={new Date(participant.signed_up_on).toLocaleDateString()}
          />
        </LabelAndItem>
      </View>
    </View>
  );
}

function SubLabelItem({ title, value }: { title: string; value: string | undefined }) {
  if (!value) {
    return null;
  }
  return (
    <View style={ContactItemStyles.container}>
      <Text style={ContactItemStyles.label}>{title}</Text>
      <Text selectable={true} dataDetectorType={'all'}>
        {value}
      </Text>
    </View>
  );
}
const ContactItemStyles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  containerInline: {
    flexDirection: 'row',
  },
  label: {
    fontWeight: 'bold',
  },
});
function InlineSubLabelItem({ title, value }: { title: string; value: string | undefined }) {
  if (!value) {
    return null;
  }
  return (
    <View style={ContactItemStyles.containerInline}>
      <Text style={ContactItemStyles.label}>{title}: </Text>
      <Text selectable={true} dataDetectorType={'all'}>
        {value}
      </Text>
    </View>
  );
}
