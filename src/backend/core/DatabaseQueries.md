## Find All Participants with no goals set

Finds all participants that do not have any goals in the current database

```sql
SELECT participants.id, participants.first_name, participants.last_name, participants.program, locations.name as location
FROM participants
         LEFT JOIN locations ON participants.location = locations.id
where (SELECT COUNT(*) FROM participant_goals where participant_goals.participant_id = participants.id) = 0;
```

## Find All Participants with no medications set



```sql
SELECT participants.id, participants.first_name, participants.last_name, participants.program, locations.name as location
FROM participants
         LEFT JOIN locations ON participants.location = locations.id
where (SELECT COUNT(*) FROM participant_medications where participant_medications.participant_id = participants.id) = 0;
```
