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
              email
            }
          }
        `}>
        {({ loading, error, data }) => {
          if (loading) return <p>Loading...</p>;
          if (error) return <p>Error :(</p>;
          const { user } = data;
          if (user) {
            return (
              <StyledHeader>
                <H1>{user.email}</H1>
                <A href="/logout">Log out</A>
              </StyledHeader>
            );
          } else {
            return (
              <StyledHeader>
                <H1>Anonymous</H1>
                <A href="/oauth/google/start">Log in</A>
              </StyledHeader>
            )
          }
        }}
      </Query>
    );
  }
}
