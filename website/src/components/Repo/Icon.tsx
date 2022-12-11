import styled from '@emotion/styled';
import { useQuery } from 'react-query';

export type IconInfo =
  | {
      type: 'png' | 'jpeg';
      size: string;
    }
  | {
      type: 'ico';
      sizes: string[];
    }
  | { type: 'svg' };

export type Icon = IconInfo & {
  url: string;
  headers: Record<string, string>;

  kind:
    | 'icon_field'
    | 'user_avatar'
    | 'app_icon'
    | 'blob'
    | 'readme_image'
    | 'site_logo'
    | 'site_favicon';
};

const StyledIcon = styled.button<{ selected: boolean }>`
  position: relative;
  border-radius: 16px;
  padding: 17px;
  border: 4px solid transparent;
  border-color: ${(props) => (props.selected ? `#007aff` : 'transparent')};
  background: ${(props) => (props.selected ? `#007aff21` : `transparent`)};
  cursor: ${(props) => (props.selected ? `auto` : `pointer`)};

  &:hover {
    background: ${(props) => (props.selected ? `#007aff21` : `#ffffff21`)};
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    aspect-ratio: 1;
    object-fit: contain;
  }

  &:empty {
    display: none;
  }
`;

const Resolution = styled.div`
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  font-family: system-ui;
  font-size: 10px;
  font-weight: 600;
  margin-bottom: 3px;
  padding: 2px 5px;
  line-height: 9px;
  color: #ffffff6b;
  background: #ffffff12;
`;

type IconProps = Icon & {
  selected?: boolean;
};

export function Icon({ url, headers, type, selected = false }: IconProps) {
  const hasHeaders = Object.keys(headers).length !== 0;

  if (hasHeaders) {
    const { data } = useQuery([url, headers], () =>
      fetch(url, { headers })
        .then((res) => res.blob())
        .then(URL.createObjectURL)
    );

    url = data!;
  }

  return (
    <StyledIcon selected={selected} onClick={() => {}}>
      <img
        src={url}
        ref={(elem: HTMLImageElement) => {
          if (!elem) return;

          function setResolution() {
            (elem.nextElementSibling as HTMLDivElement).innerText =
              type === 'svg'
                ? `SVG`
                : `${elem.naturalWidth}x${elem.naturalHeight}`;
          }

          setResolution();
          elem.onload = setResolution;
        }}
      />

      <Resolution />
    </StyledIcon>
  );
}
