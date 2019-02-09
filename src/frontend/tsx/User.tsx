import React, { PureComponent } from 'react';

import { Query } from 'react-apollo';
import { H1, P, A } from './shared/Text';
import gql from 'graphql-tag';
import _ from 'lodash';

export default class User extends PureComponent {
  render() {
    return (
      <Query
        query={gql`
          {
            user {
              moods {
                time,
                mood,
                note
              }
            }
          }
        `}>
        {({ loading, error, data }) => {
          if (loading) return <p>Loading...</p>;
          if (error) return <p>Error :(</p>;

          const { user } = data;
          const moods: { time: string, mood: number, note: string }[] = _.reverse(_.sortBy(user.moods, ['time']));
          if (user)
            return (
              <div>
                {moods.map(
                  ({time, mood, note}) => <div key={time}>
                    <P>{time} - {mood}</P>
                    <P>{note}</P>
                  </div>
                )}
              </div>
            );
          return (
            <div>
              <P>
                Anonymous
              </P>
            </div>
          );
        }}
      </Query>
    );
  }
}
