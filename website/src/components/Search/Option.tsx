import styled from '@emotion/styled';
import { IconButton } from '../IconButton';
import { OptionProps } from 'react-select';
import { Package } from './Search';

interface StyledOptionProps {
  focused: 1 | 0;
}

const StyledOption = styled(IconButton)<StyledOptionProps>`
  background: ${(props) => (props.focused ? 'red' : 'none')};

  > img {
    height: 40px;
    width: 40px;
  }
`;

export function Option({
  data,
  isDisabled,
  isFocused,
  innerRef,
  innerProps,
}: OptionProps<Package>) {
  return (
    <StyledOption
      key={data.name}
      ref={innerRef as any}
      type="npm"
      slug={data.name}
      focused={isFocused ? 1 : 0}
      aria-disabled={isDisabled}
      {...(innerProps as {})}
      onMouseDown={(e) => {
        if (e.button === 1) {
          e.preventDefault();
          window.open(e.currentTarget.href);
        }
      }}
    >
      {data.name}
    </StyledOption>
  );
}
