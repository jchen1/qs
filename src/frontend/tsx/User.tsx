import React, { PureComponent } from 'react';

import { Query } from 'react-apollo';
import { H1, P, A } from './shared/Text';
import gql from 'graphql-tag';

export default class User extends PureComponent {
  render() {
    return (
      <Query
        query={gql`
          {
            user {
              distances(startTime: "2019-01-20T00:00:01+00:00") {
                time,
                count
              }
            }
          }
        `}>
        {({ loading, error, data }) => {
          if (loading) return <p>Loading...</p>;
          if (error) return <p>Error :(</p>;

          const { user } = data;
          const distances: { time: string, count: number }[] = user.distances;
          if (user)
            return (
              <div>
                {distances.map(
                  ({time, count}) => <P key={time}>{time} - {count}</P>
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
