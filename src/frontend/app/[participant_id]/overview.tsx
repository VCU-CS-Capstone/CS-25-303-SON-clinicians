import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import LabelAndItem from '~/components/LabelAndItem';

import ProtectedRoute from '~/components/ProtectedRoute';
import { HamburgerMenu } from '~/components/menus/hamburger';
import { HamburgerOption } from '~/components/menus/hamburger/HamburgerOption';
import { ShowInsurances } from '~/components/participant/HealthInsurance';
import api from '~/lib/api';

import { StyleSheet } from 'react-native';
import {
  Participant,
  ParticipantDemographics,
  ParticipantHealthOverview,
  ParticipantStatus,
} from '~/lib/types/participant';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const router = useRouter();
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [participantHealthOverview, setParticipantHealthOverview] = useState<
    ParticipantHealthOverview | undefined
  >(undefined);
  const [participantDemographics, setParticipantDemographics] = useState<
    ParticipantDemographics | undefined
  >(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(participantNumberId);
      const healthOverview = await api.participants.fetchHealthOverview(participantNumberId);
      const demographics = await api.participants.fetchDemographic(participantNumberId);
      setParticipant(patient);
      setParticipantHealthOverview(healthOverview);
      setParticipantDemographics(demographics);
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
        <ParticipantView
          participant={participant}
          healthOverview={participantHealthOverview}
          demographics={participantDemographics}
        />
      </View>
    </ProtectedRoute>
  );
}
function ParticipantView({
  participant,
  healthOverview,
  demographics,
}: {
  participant: Participant | undefined;
  healthOverview: ParticipantHealthOverview | undefined;
  demographics: ParticipantDemographics | undefined;
}) {
  if (!participant) {
    // TODO: 404 page
    return null;
  }
  return (
    <View>
      <View style={styles.fullWidth}>
        <Text style={styles.participantName}>
          {participant.first_name} {participant.last_name}
        </Text>
      </View>
      <View style={styles.flexRowWrap}>
        <ParticipantBox {...participant} />
        <HealthOverviewBox healthOverview={healthOverview} />
        <DemographicsBox demographics={demographics} />
      </View>
    </View>
  );
}
function BoxHeader({ title }: { title: string }) {
  return (
    <View style={styles.boxHeader}>
      <Text style={styles.boxHeaderText}>{title}</Text>
      <HamburgerMenu iconWidth={36} iconHeight={36}>
        <HamburgerOption title="Open Page" />
        <HamburgerOption title="Edit Page" />
      </HamburgerMenu>
    </View>
  );
}
function ParticipantBox(participant: Participant) {
  return (
    <View style={styles.box}>
      <BoxHeader title="Participant" />
      <View style={styles.marginBottom}>
        <LabelAndItem label="Contact Info">
          <Text>{participant.phone_number_one}</Text>
        </LabelAndItem>
        <LabelAndItem label="Status">
          <Text>{ParticipantStatus.title(participant.status)}</Text>
        </LabelAndItem>
      </View>
    </View>
  );
}

function HealthOverviewBox({
  healthOverview,
}: {
  healthOverview: ParticipantHealthOverview | undefined;
}) {
  if (!healthOverview) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  }
  return (
    <View style={styles.box}>
      <BoxHeader title="Health Overview" />
      <View>
        <YesOrNo
          label="Takes More Than 5 Medications"
          value={healthOverview.takes_more_than_5_medications}
        />
        <YesOrNo
          label="Has Blood Pressure Cuff"
          value={healthOverview.takes_more_than_5_medications}
        />
      </View>
      <LabelAndItem label="Allergies">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
      <LabelAndItem label="Reported Health Conditions">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
    </View>
  );
}
function YesOrNo({ label, value }: { label: string; value?: boolean }) {
  return (
    <View style={styles.marginBottom}>
      <Text style={styles.yesOrNoText}>
        {label}: {value ? 'Yes' : 'No'}
      </Text>
    </View>
  );
}
function DemographicsBox({ demographics }: { demographics: ParticipantDemographics | undefined }) {
  if (!demographics) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  }
  return (
    <View style={styles.halfWidthBox}>
      <BoxHeader title="Demographics" />

      <View style={styles.flexRowBetween}>
        <LabelAndItem label="Age">
          <Text>{demographics.age}</Text>
        </LabelAndItem>
        <YesOrNo label="Is Veteran" value={demographics.is_veteran} />
      </View>
      <LabelAndItem label="Health Insurace">
        <ShowInsurances insurances={demographics.health_insurance || []} />
      </LabelAndItem>
    </View>
  );
}

const styles = StyleSheet.create({
  fullWidth: {
    width: '100%',
  },
  participantName: {
    fontSize: 32,
    fontWeight: 'bold',
  },
  flexRowWrap: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  boxHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    borderBottomWidth: 1,
    padding: 16,
  },
  boxHeaderText: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  box: {
    flexBasis: '50%',
    borderWidth: 1,
    padding: 16,
  },
  marginBottom: {
    marginBottom: 16,
  },
  yesOrNoText: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  halfWidthBox: {
    width: '50%',
    borderWidth: 1,
    padding: 16,
  },
  flexRowBetween: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
});
