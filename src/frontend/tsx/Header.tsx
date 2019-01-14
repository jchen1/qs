import React, { PureComponent } from 'react';
import styled from 'styled-components';

import { Query } from 'react-apollo';
import { H1, P, A } from './shared/Text';
import gql from 'graphql-tag';

const StyledHeader = styled.div`
  display: flex;
  grid-row-start: header;
  grid-row-end: content;
`;

export default class Header extends PureComponent {
  render() {
    return (
      <Query
        query={gql`
          {
            user {
              id
              email
              name
            }
          }
        `}>
        {({ loading, error, data }) => {
          if (loading) return <p>Loading...</p>;
          if (error) return <p>Error :(</p>;
          const { name } = data.user;
          return (
            <StyledHeader>
              <H1>{name}</H1>
            </StyledHeader>
          );
          // if (loading) return <p>Loading...</p>;
          // if (error) return <p>Error :(</p>;

          // const { user } = data;
          // if (user)
          //   return (
          //     <div>
          //       <H1>{data.user.email}</H1>
          //       <P>{data.user.id}</P>
          //       <A href="oauth/fitbit/start">Fitbit</A>
          //       <A href="logout">Logout</A>
          //     </div>
          //   );
          // return (
          //   <div>
          //     <p>
          //       Anonymous - <a href="oauth/google/start">Login</a>
          //     </p>
          //   </div>
          // );
        }}
      </Query>
    );
  }
}
