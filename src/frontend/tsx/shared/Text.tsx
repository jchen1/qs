import styled from 'styled-components';

import { violet, orange, titleFont, bodyFont, beige, blue, red } from '../constants';

export const H1 = styled.h1`
  color: ${orange};
  font-family: ${titleFont};
  font-size: 2em;
`;

export const P = styled.p`
  color: ${beige};
  font-family: ${bodyFont};
`;

export const A = styled.a`
  color: ${red};
  font-family: ${bodyFont};
  text-decoration: none;
`;
