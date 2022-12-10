import { keyframes } from '@emotion/react';
import styled from '@emotion/styled';

const animate = keyframes`
  0% {
    transform: translateY(0) rotate(0deg);
    opacity: 1;
    border-radius: 0;
  }

  100% {
    transform: translateY(-100vh) rotate(720deg);
    opacity: 0;
    border-radius: 50%;
  }
`;

const Circles = styled.ul`
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: -1;
  margin: 0;
`;

const Circle = styled.li`
  position: absolute;
  display: block;
  list-style: none;
  width: 20px;
  height: 20px;
  background: rgba(255, 255, 255, 0.1);
  animation: ${animate} 25s linear infinite;
  bottom: -150px;

  &:nth-of-type(1) {
    left: 25%;
    width: 80px;
    height: 80px;
    animation-delay: 0s;
  }

  &:nth-of-type(2) {
    left: 10%;
    width: 20px;
    height: 20px;
    animation-delay: 2s;
    animation-duration: 12s;
  }

  &:nth-of-type(3) {
    left: 70%;
    width: 20px;
    height: 20px;
    animation-delay: 4s;
  }

  &:nth-of-type(4) {
    left: 40%;
    width: 60px;
    height: 60px;
    animation-delay: 0s;
    animation-duration: 18s;
  }

  &:nth-of-type(5) {
    left: 65%;
    width: 20px;
    height: 20px;
    animation-delay: 0s;
  }

  &:nth-of-type(6) {
    left: 75%;
    width: 110px;
    height: 110px;
    animation-delay: 3s;
  }

  &:nth-of-type(7) {
    left: 35%;
    width: 150px;
    height: 150px;
    animation-delay: 7s;
  }

  &:nth-of-type(8) {
    left: 50%;
    width: 25px;
    height: 25px;
    animation-delay: 15s;
    animation-duration: 45s;
  }

  &:nth-of-type(9) {
    left: 20%;
    width: 15px;
    height: 15px;
    animation-delay: 2s;
    animation-duration: 35s;
  }

  &:nth-of-type(10) {
    left: 85%;
    width: 150px;
    height: 150px;
    animation-delay: 0s;
    animation-duration: 11s;
  }
`;

export function BackgroundEffect() {
  return (
    <Circles>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
      <Circle></Circle>
    </Circles>
  );
}
